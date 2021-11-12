#!/bin/bash

if [[ $# -le 1 ]]
then
    echo "Usage: $0 BASE_DIRECTORY [usa|jp]"
    exit 1
fi

DIR=$1
VERSION=$2

FUNCTIONS="${DIR}/tables/${VERSION}/functions.csv"
VARIABLES="${DIR}/tables/${VERSION}/variables.csv"

BASEROM_DIR="${DIR}/baserom/${VERSION}"
ASM_DIR="${DIR}/asm/${VERSION}"
TABLES_DIR="${DIR}/tables/${VERSION}"
CONTEXT_DIR="${DIR}/context/${VERSION}"

./simpleDisasm.py "${BASEROM_DIR}/boot.bin" "${ASM_DIR}/boot" --vram 80000450 --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_boot.csv" --save-context "${CONTEXT_DIR}/boot.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_B3C70.bin" "${ASM_DIR}/file_B3C70" --vram 8010AAF0 --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_B3C70.csv" --save-context "${CONTEXT_DIR}/B3C70.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_DCF60.bin" "${ASM_DIR}/file_DCF60" --vram 80133DE0 --functions ${FUNCTIONS} --variables ${VARIABLES} --save-context "${CONTEXT_DIR}/DCF60.txt"
