import json
import re
import requests
import struct
from time import sleep
from eth_utils import from_wei, to_wei


class bcolors:
    HEADER = '\033[95m'
    RED = '\033[31m'
    GRAY = '\033[37m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'


ALICE = ("8fd379246834eac74b8419ffda202cf8051f7a03", "Alice")
BOB = ("88f9b82462f6c4bf4a0fb15e5c3971559a316e7f", "Bob")
CHARLIE = ("e8acf143afbf8b1371a20ea934d334180190eac1", "Charlie")


def get_balance_from_query_res(res):
    return struct.unpack(">d", bytes(eval(re.findall(r"value: \[.*\]", res.text)[0].split(": ")[1])))[0]

def query_balance(port, ident, debug=True):
    if debug:
        print(f"{bcolors.RED}Querying balance for {ident} from replica #{port} ...{bcolors.ENDC}")

    res = requests.get(f"http://localhost:{port}/abci_query", params={
        "data": ident[0],
        "path": "",
    })

    if debug:
        print("#", res.request.url)

    balance = from_wei(get_balance_from_query_res(res), 'ether')

    if debug:
        print(f"{bcolors.RED} -> {ident[1]}'s balance: {balance} ETH{bcolors.ENDC}")
        print()

    return balance

def query_balances(port):
    balance_A = query_balance(port, ALICE, False)
    balance_B = query_balance(port, BOB, False)
    balance_C = query_balance(port, CHARLIE, False)
    print(f"{bcolors.OKGREEN}Balances at replica #{port}:  Alice {balance_A} ETH  /  Bob {balance_B} ETH  /  Charlie {balance_C} ETH{bcolors.ENDC}")

def make_payment(port, frm_ident, to_ident, amt):
    print(f"{bcolors.RED}Issuing payment at replica #{port}: {frm_ident} -> {to_ident} ({amt} ETH){bcolors.ENDC}")
    res = requests.get(f"http://localhost:{port}/broadcast_tx", params={
        "tx": requests.utils.quote(json.dumps({
            "from": frm_ident[0],
            "to": to_ident[0],
            "value": to_wei(amt, 'ether'),
            "gas": 40000,
            "gas_price": 875000000,
            "data": ""
        }))
    })
    print("#", res.request.url)
    print()


print()
input("Let's look at the initial balances ...   <ENTER to continue>")
print()

query_balance(3002, ALICE)
query_balance(3002, BOB)
query_balance(3002, CHARLIE)

print()
input("Let's issue a double spend from Alice ...   <ENTER to continue>")
print()

make_payment(3009, ALICE, BOB, 0.6)
make_payment(3016, ALICE, CHARLIE, 0.6)

print()
print("Look, so far neither has been reached consensus upon:")
print()

query_balances(3002)
query_balances(3009)
query_balances(3016)
query_balances(3023)
print()

print()
input("Let's wait a bit and check again ...   <ENTER to continue>")
print()

query_balances(3002)
query_balances(3009)
query_balances(3016)
query_balances(3023)
print()
