#!/bin/bash

PEM=`cat ./identity.pem`

for f in `ls -1 ./commands/`; do
    out=${f/sh/txt}
    echo "$PEM" | sh "commands/$f" > "./outputs/$out"
done
