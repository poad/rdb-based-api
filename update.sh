#!/bin/sh

CUR=$(pwd)

CURRENT=$(cd "$(dirname "$0")" || exit;pwd)
echo "${CURRENT}"

cd "${CURRENT}" || exit

if ! (git pull --prune); then
  cd "${CUR}" || exit
  exit 1
fi

if ! (disable-checkout-persist-credentials); then
  cd "${CUR}" || exit
  exit 1
fi

set -- "mysql-based/sqlx-based"
for target in "$@"; do
  if ! (cd "${CURRENT}/${target}" || exit && cargo update); then
    cd "${CUR}" || exit
    exit 1
  fi
  echo ""
  pwd
done

if ! (cd "${CURRENT}" || exit && git add . && git commit -am "Bumps crates" && git push); then
  cd "${CUR}" || exit
  exit 1
fi

cd "${CUR}" || exit
