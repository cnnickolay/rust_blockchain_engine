#!/bin/bash

if [ -z ${REMOTE_VALIDATOR} ];
then
    node --host $VALIDATOR_CALLBACK --port 9065 --private-key $(</test-data/$VALIDATOR_SK) --public-key $(</test-data/$VALIDATOR_PK)
else
    node --host $VALIDATOR_CALLBACK --port 9065 --private-key $(</test-data/$VALIDATOR_SK) --public-key $(</test-data/$VALIDATOR_PK) --remote-validator $REMOTE_VALIDATOR:9065
fi

