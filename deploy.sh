#!/bin/bash

set -e
trunk build
cd dist
git init
git add -A
git commit -m 'deploy'
git push -f git@github.com:Akiko97/web-tester.git main:gh-pages
cd -
