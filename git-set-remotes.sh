#!/bin/sh

# Credits: http://stackoverflow.com/a/750191

git remote remove origin
git remote add origin ssh://git@github.com/RandomAmbersky/NationOfLastLand-ai
git push --set-upstream origin main
#chmod 600 ~/.ssh/ed_wanderer
