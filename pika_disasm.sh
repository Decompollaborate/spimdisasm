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
DATA_DIR="${DIR}/data/${VERSION}"
TABLES_DIR="${DIR}/tables/${VERSION}"
CONTEXT_DIR="${DIR}/context/${VERSION}"

./simpleDisasm.py "${BASEROM_DIR}/boot.bin"        "${ASM_DIR}/boot"        --data-output "${DATA_DIR}/boot"        --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_boot.csv"        --save-context "${CONTEXT_DIR}/boot.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_B3C70.bin"  "${ASM_DIR}/file_B3C70"  --data-output "${DATA_DIR}/file_B3C70"  --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_B3C70.csv"  --save-context "${CONTEXT_DIR}/B3C70.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_DCF60.bin"  "${ASM_DIR}/file_DCF60"  --data-output "${DATA_DIR}/file_DCF60"  --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_DCF60.csv"  --save-context "${CONTEXT_DIR}/DCF60.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_107190.bin" "${ASM_DIR}/file_107190" --data-output "${DATA_DIR}/file_107190" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_107190.csv" --save-context "${CONTEXT_DIR}/107190.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_10F7F0.bin" "${ASM_DIR}/file_10F7F0" --data-output "${DATA_DIR}/file_10F7F0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_10F7F0.csv" --save-context "${CONTEXT_DIR}/10F7F0.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_1426B0.bin" "${ASM_DIR}/file_1426B0" --data-output "${DATA_DIR}/file_1426B0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_1426B0.csv" --save-context "${CONTEXT_DIR}/1426B0.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_182B40.bin" "${ASM_DIR}/file_182B40" --data-output "${DATA_DIR}/file_182B40" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_182B40.csv" --save-context "${CONTEXT_DIR}/182B40.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_19C470.bin" "${ASM_DIR}/file_19C470" --data-output "${DATA_DIR}/file_19C470" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_19C470.csv" --save-context "${CONTEXT_DIR}/19C470.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_1C5E50.bin" "${ASM_DIR}/file_1C5E50" --data-output "${DATA_DIR}/file_1C5E50" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_1C5E50.csv" --save-context "${CONTEXT_DIR}/1C5E50.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_1FE3D0.bin" "${ASM_DIR}/file_1FE3D0" --data-output "${DATA_DIR}/file_1FE3D0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_1FE3D0.csv" --save-context "${CONTEXT_DIR}/1FE3D0.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_2535D0.bin" "${ASM_DIR}/file_2535D0" --data-output "${DATA_DIR}/file_2535D0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_2535D0.csv" --save-context "${CONTEXT_DIR}/2535D0.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_29CA40.bin" "${ASM_DIR}/file_29CA40" --data-output "${DATA_DIR}/file_29CA40" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_29CA40.csv" --save-context "${CONTEXT_DIR}/29CA40.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_2BD950.bin" "${ASM_DIR}/file_2BD950" --data-output "${DATA_DIR}/file_2BD950" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_2BD950.csv" --save-context "${CONTEXT_DIR}/2BD950.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_2DA640.bin" "${ASM_DIR}/file_2DA640" --data-output "${DATA_DIR}/file_2DA640" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_2DA640.csv" --save-context "${CONTEXT_DIR}/2DA640.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_31AA90.bin" "${ASM_DIR}/file_31AA90" --data-output "${DATA_DIR}/file_31AA90" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_31AA90.csv" --save-context "${CONTEXT_DIR}/31AA90.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_3728F0.bin" "${ASM_DIR}/file_3728F0" --data-output "${DATA_DIR}/file_3728F0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_3728F0.csv" --save-context "${CONTEXT_DIR}/3728F0.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_3C8950.bin" "${ASM_DIR}/file_3C8950" --data-output "${DATA_DIR}/file_3C8950" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_3C8950.csv" --save-context "${CONTEXT_DIR}/3C8950.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_3E3D00.bin" "${ASM_DIR}/file_3E3D00" --data-output "${DATA_DIR}/file_3E3D00" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_3E3D00.csv" --save-context "${CONTEXT_DIR}/3E3D00.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_3F6EF0.bin" "${ASM_DIR}/file_3F6EF0" --data-output "${DATA_DIR}/file_3F6EF0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_3F6EF0.csv" --save-context "${CONTEXT_DIR}/3F6EF0.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_417E50.bin" "${ASM_DIR}/file_417E50" --data-output "${DATA_DIR}/file_417E50" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_417E50.csv" --save-context "${CONTEXT_DIR}/417E50.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_430E20.bin" "${ASM_DIR}/file_430E20" --data-output "${DATA_DIR}/file_430E20" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_430E20.csv" --save-context "${CONTEXT_DIR}/430E20.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_44E2E0.bin" "${ASM_DIR}/file_44E2E0" --data-output "${DATA_DIR}/file_44E2E0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_44E2E0.csv" --save-context "${CONTEXT_DIR}/44E2E0.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_456EF0.bin" "${ASM_DIR}/file_456EF0" --data-output "${DATA_DIR}/file_456EF0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_456EF0.csv" --save-context "${CONTEXT_DIR}/456EF0.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_47EBD0.bin" "${ASM_DIR}/file_47EBD0" --data-output "${DATA_DIR}/file_47EBD0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_47EBD0.csv" --save-context "${CONTEXT_DIR}/47EBD0.txt"
./simpleDisasm.py "${BASEROM_DIR}/file_4A49D0.bin" "${ASM_DIR}/file_4A49D0" --data-output "${DATA_DIR}/file_4A49D0" --functions ${FUNCTIONS} --variables ${VARIABLES} --file-splits "${TABLES_DIR}/files_file_4A49D0.csv" --save-context "${CONTEXT_DIR}/4A49D0.txt"

