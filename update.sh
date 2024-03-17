#!/bin/sh

CUR=$(pwd)

CURRENT=$(cd "$(dirname "$0")" || exit;pwd)
echo "${CURRENT}"

cd "${CURRENT}" || exit
git pull --prune
result=$?
if [ $result -ne 0 ]; then
  cd "${CUR}" || exit
  exit $result
fi

set -- "mysql-based"
for target in "$@"; do
  cd "${CURRENT}/${target}" || exit
  result=$?
  if [ $result -ne 0 ]; then
    cd "${CUR}" || exit
    exit $result
  fi
  echo ""
  pwd
  cargo update
  result=$?
  if [ $result -ne 0 ]; then
    cd "${CUR}" || exit
    exit $result
  fi
done

cd "${CURRENT}" || exit
result=$?
if [ $result -ne 0 ]; then
  cd "${CUR}" || exit
  exit $result
fi
git add . && git commit -am "Bumps crates" && git push
result=$?
if [ $result -ne 0 ]; then
  cd "${CUR}" || exit
  exit $result
fi

cd "${CUR}" || exit
