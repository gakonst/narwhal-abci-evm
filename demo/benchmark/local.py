# Copyright(C) Facebook, Inc. and its affiliates.
import subprocess
from math import ceil
from os.path import basename, splitext
from time import sleep

from benchmark.commands import CommandMaker
from benchmark.config import Key, LocalCommittee, NodeParameters, BenchParameters, ConfigError
from benchmark.logs import LogParser, ParseError
from benchmark.utils import Print, BenchError, PathMaker


class LocalBench:
    BASE_PORT = 3000

    def __init__(self, bench_parameters_dict, node_parameters_dict):
        print("bench params dict", bench_parameters_dict)
        print("node params dict", node_parameters_dict)
        try:
            self.bench_parameters = BenchParameters(bench_parameters_dict)
            self.node_parameters = NodeParameters(node_parameters_dict)
        except ConfigError as e:
            raise BenchError('Invalid nodes or bench parameters', e)

    def __getattr__(self, attr):
        return getattr(self.bench_parameters, attr)

    def _background_run(self, command, log_file):
        name = splitext(basename(log_file))[0]
        cmd = f'{command} &> {log_file}'
        # cmd = f'{command}'
        print("Background run:", ['tmux', 'new', '-d', '-s', name, cmd])
        subprocess.run(['tmux', 'new', '-d', '-s', name, cmd], check=True)

    def _kill_nodes(self):
        # try:
        cmd = CommandMaker.kill().split()
        subprocess.run(cmd)#, stderr=subprocess.DEVNULL)
        # except subprocess.SubprocessError as e:
        #     raise BenchError('Failed to kill testbed', e)

    def run(self, debug=False):
        assert isinstance(debug, bool)
        Print.heading('Starting local benchmark')

        # Kill any previous testbed.
        self._kill_nodes()

        try:
            Print.info('Setting up testbed...')
            nodes, rate = self.nodes[0], self.rate[0]

            # Cleanup all files.
            cmd = f'{CommandMaker.clean_logs()} ; {CommandMaker.cleanup()}'
            subprocess.run([cmd], shell=True, stderr=subprocess.DEVNULL)
            # sleep(0.5)  # Removing the store may take time.

            print(cmd)

            # Recompile the latest code.
            cmd = CommandMaker.compile().split()
            print(cmd)
            subprocess.run(cmd, check=True, cwd=PathMaker.node_crate_path())

            # Create alias for the client and nodes binary.
            cmd = CommandMaker.alias_binaries(PathMaker.binary_path())
            print(cmd)
            subprocess.run([cmd], shell=True)

            # Generate configuration files.
            keys = []
            key_files = [PathMaker.key_file(i) for i in range(nodes)]
            for filename in key_files:
                cmd = CommandMaker.generate_key(filename).split()
                subprocess.run(cmd, check=True)
                keys += [Key.from_file(filename)]

            print(key_files, keys)

            names = [x.name for x in keys]
            committee = LocalCommittee(names, self.BASE_PORT, self.workers)
            # prints to .committee.json
            # print(PathMaker.committee_file())
            committee.print(PathMaker.committee_file())

            print(names, committee)


            self.node_parameters.print(PathMaker.parameters_file())

            # Run the clients (they will wait for the nodes to be ready).
            # Worker transaction endpoint (3003, 3008 etc.)
            # Probably the TPU equivalent?
            workers_addresses = committee.workers_addresses(self.faults)


            print("[+] Spinning up apps")
            # Run the apps
            for i, address in enumerate(committee.app_addresses(self.faults)):
                cmd = CommandMaker.run_app(address)
                log_file = PathMaker.app_log_file(i)
                # Each one of these starts a new tmux session
                self._background_run(cmd, log_file)

            sleep(1)

            # print("[+] Spinning up clients")
            # # The benchmark clients connect to the worker addresses to submit transactions
            # # Starts 1 client for each worker process.
            # rate_share = ceil(rate / committee.workers())
            # for i, addresses in enumerate(workers_addresses):
            #     for (id, address) in addresses:
            #         cmd = CommandMaker.run_client(
            #             address,
            #             self.tx_size,
            #             rate_share,
            #             [x for y in workers_addresses for _, x in y]
            #         )
            #         log_file = PathMaker.client_log_file(i, id)
            #         print("--> [+] Running", cmd, log_file)
            #         self._background_run(cmd, log_file)


            print("[+] Spinning up primaries")
            # Run the primaries (except the faulty ones).
            for i, address in enumerate(committee.primary_addresses(self.faults)):
                cmd = CommandMaker.run_primary(
                    PathMaker.key_file(i),
                    PathMaker.committee_file(),
                    PathMaker.db_path(i),
                    PathMaker.parameters_file(),
                    app_api = committee.app_addresses(self.faults)[i],
                    abci_api = committee.rpc_addresses(self.faults)[i],
                    debug=debug
                )
                log_file = PathMaker.primary_log_file(i)
                # Each one of these starts a new tmux session
                self._background_run(cmd, log_file)


            print("[+] Spinning up workers")
            # Run the workers (except the faulty ones).
            for i, addresses in enumerate(workers_addresses):
                for (id, address) in addresses:
                    cmd = CommandMaker.run_worker(
                        PathMaker.key_file(i),
                        PathMaker.committee_file(),
                        PathMaker.db_path(i, id),
                        PathMaker.parameters_file(),
                        id,  # The worker's id.
                        debug=debug
                    )
                    log_file = PathMaker.worker_log_file(i, id)
                    self._background_run(cmd, log_file)


            # Wait for all transactions to be processed.
            Print.info(f'Running benchmark ({self.duration} sec)...')
            sleep(self.duration)
            self._kill_nodes()

            # # Parse logs and return the parser.
            # Print.info('Parsing logs...')
            # return LogParser.process(PathMaker.logs_path(), faults=self.faults)

        except Exception as e:
            self._kill_nodes()
            raise BenchError('Failed to run benchmark', e)
