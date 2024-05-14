#/bin/sh

set +xe

function check_variable {
    if [ -z ${2+x} ]; then
        echo variable "$1" not found
        echo aborting
        exit 1
    else
        echo variable "$1" found, set to $2
    fi
}

function commands_check {
    command -v gcc
}

check_variable CC "$CC"
check_variable CFLAGS "$CFLAGS"
check_variable DIFF "$DIFF"
check_variable DIFFFLAGS "$DIFFFLAGS"
check_variable PROBLEMS "$PROBLEMS"

for p in "$PROBLEMS"; do

done