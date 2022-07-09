# Copyright(C) Facebook, Inc. and its affiliates.
from os.path import join
import os

from benchmark.utils import PathMaker

NODE = "./../target/debug/node"


class CommandMaker:

    @staticmethod
    def cleanup():
        return (
            f'rm -r .db-* ; rm .*.json ; mkdir -p {PathMaker.results_path()}'
        )

    @staticmethod
    def clean_logs():
        return f'rm -r {PathMaker.logs_path()} ; mkdir -p {PathMaker.logs_path()}'

    @staticmethod
    def compile():
        return 'cargo build'
        # return 'cargo build --quiet --release --features benchmark'

    @staticmethod
    def generate_key(filename):
        assert isinstance(filename, str)
        return f'{NODE} generate_keys --filename {filename}'

    @staticmethod
    def run_primary(keys, committee, store, parameters, app_api, abci_api, debug=False):
        print(store, keys)
        assert isinstance(keys, str)
        assert isinstance(committee, str)
        assert isinstance(parameters, str)
        assert isinstance(debug, bool)
        v = '-vvv' if debug else '-vv'
        return (f'{NODE} {v} run --keys {keys} --committee {committee} '
                f'--store {store} --parameters {parameters} primary --app-api {app_api} --abci-api {abci_api} ')

    @staticmethod
    def run_worker(keys, committee, store, parameters, id, debug=False):
        assert isinstance(keys, str)
        assert isinstance(committee, str)
        assert isinstance(parameters, str)
        assert isinstance(debug, bool)
        v = '-vvv' if debug else '-vv'
        return (f'{NODE} {v} run --keys {keys} --committee {committee} '
                f'--store {store} --parameters {parameters} worker --id {id}')

    @staticmethod
    def run_client(address, size, rate, nodes):
        assert isinstance(address, str)
        assert isinstance(size, int) and size > 0
        assert isinstance(rate, int) and rate >= 0
        assert isinstance(nodes, list)
        assert all(isinstance(x, str) for x in nodes)
        nodes = f'--nodes {" ".join(nodes)}' if nodes else ''
        return f'./../target/debug/benchmark_client {address} --size {size} --rate {rate} {nodes}'

    @staticmethod
    def run_app(listen_on):
        assert isinstance(listen_on, str)
        return f'../target/debug/evm-app --demo {listen_on}'

    @staticmethod
    def kill():
        print("os.getenv('TMUX'):", os.getenv('TMUX'))
        if os.getenv('TMUX'):
            # running within tmux (Georgios' config)
            # kill all other sessions
            return "tmux kill-session -a"
        else:
            # running without tmux (Joachim's config)
            # This does not work when running in Tmux
            return 'tmux kill-server'

    @staticmethod
    def alias_binaries(origin):
        assert isinstance(origin, str)
        # This is aliasing only the release
        # print('Origin', origin)
        node, client = join(origin, 'node'), join(origin, 'benchmark_client')
        return f'rm node ; rm benchmark_client ; ln -s {node} . ; ln -s {client} .'
