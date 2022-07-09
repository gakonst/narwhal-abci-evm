#!/bin/bash -ve

curl "http://localhost:3023/abci_query?data=Alice&path="
curl "http://localhost:3023/abci_query?data=Bob&path="
curl "http://localhost:3023/abci_query?data=Charly&path="

curl "http://localhost:3002/broadcast_tx?tx=Alice,Bob,20"
curl "http://localhost:3002/broadcast_tx?tx=Alice,Charly,20"

sleep 3

curl "http://localhost:3002/abci_query?data=Alice&path="
curl "http://localhost:3002/abci_query?data=Bob&path="
curl "http://localhost:3002/abci_query?data=Charly&path="

curl "http://localhost:3009/abci_query?data=Alice&path="
curl "http://localhost:3009/abci_query?data=Bob&path="
curl "http://localhost:3009/abci_query?data=Charly&path="

curl "http://localhost:3016/abci_query?data=Alice&path="
curl "http://localhost:3016/abci_query?data=Bob&path="
curl "http://localhost:3016/abci_query?data=Charly&path="

curl "http://localhost:3023/abci_query?data=Alice&path="
curl "http://localhost:3023/abci_query?data=Bob&path="
curl "http://localhost:3023/abci_query?data=Charly&path="

