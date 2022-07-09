#!/bin/bash -ve

cat logs/primary-0.log | grep -i LEDGER | awk '{ print $7; }' > logs/primary-0-parsed.log
cat logs/primary-1.log | grep -i LEDGER | awk '{ print $7; }' > logs/primary-1-parsed.log
cat logs/primary-2.log | grep -i LEDGER | awk '{ print $7; }' > logs/primary-2-parsed.log
cat logs/primary-3.log | grep -i LEDGER | awk '{ print $7; }' > logs/primary-3-parsed.log
