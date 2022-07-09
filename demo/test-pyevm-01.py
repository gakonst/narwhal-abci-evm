import json
import re
import requests
import struct
from time import sleep
from eth_utils import from_wei, to_wei

address_to_name = {
    "8fd379246834eac74b8419ffda202cf8051f7a03": "Alice",
    "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f": "Bob",
    "e8acf143afbf8b1371a20ea934d334180190eac1": "Charlie",
}

def get_balance_from_query_res(res):
    return struct.unpack(">d", bytes(eval(re.findall(r"value: \[.*\]", res.text)[0].split(": ")[1])))[0]

def log_query_request(port, address):
    print(f"Querying balance for {address} ({address_to_name[address]}) from {port}")

def log_tx_request(port, from_address, to_address, value):
    print(f"Sent transaction to {port} where from={from_address} ({address_to_name[from_address]}), to={to_address} ({address_to_name[to_address]}), value={value} ETH")

alice_init_balance_res = requests.get("http://localhost:3002/abci_query", params={
    "data": "8fd379246834eac74b8419ffda202cf8051f7a03",
    "path": "",
})
log_query_request(3002, "8fd379246834eac74b8419ffda202cf8051f7a03")
alice_balance = from_wei(get_balance_from_query_res(alice_init_balance_res), 'ether')
print(f"Alice's initial balance:\n{alice_balance} ETH\n")

bob_init_balance_res = requests.get("http://localhost:3002/abci_query", params={
    "data": "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f",
    "path": "",
})
log_query_request(3002, "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f")
bob_balance = from_wei(get_balance_from_query_res(bob_init_balance_res), 'ether')
print(f"Bob's initial balance:\n{bob_balance} ETH\n")

charlie_init_balance_res = requests.get("http://localhost:3002/abci_query", params={
    "data": "e8acf143afbf8b1371a20ea934d334180190eac1",
    "path": "",
})
log_query_request(3002, "e8acf143afbf8b1371a20ea934d334180190eac1")
charlie_balance = from_wei(get_balance_from_query_res(charlie_init_balance_res), 'ether')
print(f"Charlie's initial balance:\n{charlie_balance} ETH\n")

print("\n---\n")
print("Alice attempts to send a transaction where she transfers 0.6 to Bob to one consensus node, and a transaction where she transfers 0.6 ETH to Charlie to another node.\n")
res = requests.get("http://localhost:3009/broadcast_tx", params={
    "tx": requests.utils.quote(json.dumps({
        "from": "8fd379246834eac74b8419ffda202cf8051f7a03",
        "to": "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f",
        "value": to_wei(0.6, 'ether'),
        "gas": 40000,
        "gas_price": 875000000,
        "data": ""
    }))
})
log_tx_request(3009, "8fd379246834eac74b8419ffda202cf8051f7a03", "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f", 0.6)
res = requests.get("http://localhost:3016/broadcast_tx", params={
    "tx": requests.utils.quote(json.dumps({
        "from": "8fd379246834eac74b8419ffda202cf8051f7a03",
        "to": "e8acf143afbf8b1371a20ea934d334180190eac1",
        "value": to_wei(0.6, 'ether'),
        "gas": 40000,
        "gas_price": 875000000,
        "data": ""
    }))
})
log_tx_request(3016, "8fd379246834eac74b8419ffda202cf8051f7a03", "e8acf143afbf8b1371a20ea934d334180190eac1", 0.6)

sleep(5)

print("\n---\n")

alice_final_balance_res_1 = requests.get("http://localhost:3009/abci_query", params={
    "data": "8fd379246834eac74b8419ffda202cf8051f7a03",
    "path": "",
})
log_query_request(3009, "8fd379246834eac74b8419ffda202cf8051f7a03")
alice_balance = from_wei(get_balance_from_query_res(alice_final_balance_res_1), 'ether')
print(f"Alice's final balance according to the 1st consensus node:\n{alice_balance} ETH\n")

bob_final_balance_res_1 = requests.get("http://localhost:3009/abci_query", params={
    "data": "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f",
    "path": "",
})
log_query_request(3009, "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f")
bob_balance = from_wei(get_balance_from_query_res(bob_final_balance_res_1), 'ether')
print(f"Bob's final balance according to the 1st consensus node:\n{bob_balance} ETH\n")

charlie_final_balance_res_1 = requests.get("http://localhost:3009/abci_query", params={
    "data": "e8acf143afbf8b1371a20ea934d334180190eac1",
    "path": "",
})
log_query_request(3009, "e8acf143afbf8b1371a20ea934d334180190eac1")
charlie_balance = from_wei(get_balance_from_query_res(charlie_final_balance_res_1), 'ether')
print(f"Charlie's final balance according to the 1st consensus node:\n{charlie_balance} ETH\n")

print("\n---\n")

alice_final_balance_res_2 = requests.get("http://localhost:3016/abci_query", params={
    "data": "8fd379246834eac74b8419ffda202cf8051f7a03",
    "path": "",
})
log_query_request(3016, "8fd379246834eac74b8419ffda202cf8051f7a03")
alice_balance = from_wei(get_balance_from_query_res(alice_final_balance_res_2), 'ether')
print(f"Alice's final balance according to the 2nd consensus node:\n{alice_balance} ETH\n")

bob_final_balance_res_2 = requests.get("http://localhost:3016/abci_query", params={
    "data": "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f",
    "path": "",
})
log_query_request(3016, "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f")
bob_balance = from_wei(get_balance_from_query_res(bob_final_balance_res_2), 'ether')
print(f"Bob's final balance according to the 2nd consensus node:\n{bob_balance} ETH\n")

charlie_final_balance_res_2 = requests.get("http://localhost:3016/abci_query", params={
    "data": "e8acf143afbf8b1371a20ea934d334180190eac1",
    "path": "",
})
log_query_request(3016, "e8acf143afbf8b1371a20ea934d334180190eac1")
charlie_balance = from_wei(get_balance_from_query_res(charlie_final_balance_res_2), 'ether')
print(f"Charlie's final balance according to the 2nd consensus node:\n{charlie_balance} ETH\n")
