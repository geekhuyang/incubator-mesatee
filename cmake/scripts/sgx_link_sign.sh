#!/bin/bash
set -e
REQUIRED_ENVS=("TOOLCHAIN_DEPS_DIR" "CMAKE_C_COMPILER" "CUR_MODULE_NAME" "CUR_MODULE_PATH"
"MESATEE_SERVICE_INSTALL_DIR" "MESATEE_OUT_DIR" "MESATEE_PROJECT_ROOT" "Service_Library_Name"
"SGX_COMMON_CFLAGS" "SGX_ENCLAVE_SIGNER" "SGX_LIBRARY_PATH" "TARGET" "Trts_Library_Name"
"TRUSTED_TARGET_DIR")
for var in "${REQUIRED_ENVS[@]}"; do
    [ -z "${!var}" ] && echo "Please set ${var}" && exit -1
done

LIBENCLAVE_PATH="${TRUSTED_TARGET_DIR}/${TARGET}/lib${CUR_MODULE_NAME}_enclave.a"
CONFIG_PATH="${MESATEE_PROJECT_ROOT}/${CUR_MODULE_PATH}/sgx_trusted_lib/Enclave.config.xml"
SIGNED_PATH="${MESATEE_SERVICE_INSTALL_DIR}/${CUR_MODULE_NAME}.enclave.signed.so"
CUR_ENCLAVE_INFO_PATH="${MESATEE_OUT_DIR}/${CUR_MODULE_NAME}_enclave_info.txt"
if [ ! "$LIBENCLAVE_PATH" -nt "$SIGNED_PATH" ] \
    && [ ! "$CONFIG_PATH" -nt "$SIGNED_PATH" ] \
    && [  ! "$SIGNED_PATH" -nt "$CUR_ENCLAVE_INFO_PATH" ]; then
    # "Skip linking ${SIGNED_PATH} because of no update."
    exit 0
fi
cd ${MESATEE_OUT_DIR}
${CMAKE_C_COMPILER} libEnclave_t.o -o \
    ${MESATEE_OUT_DIR}/${CUR_MODULE_NAME}.enclave.so ${SGX_COMMON_CFLAGS} \
    -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles \
    -L${SGX_LIBRARY_PATH} -Wl,--whole-archive -l${Trts_Library_Name} \
    -Wl,--no-whole-archive -Wl,--start-group \
    -l${Service_Library_Name} -lsgx_tprotected_fs -lsgx_tkey_exchange \
    -lsgx_tstdc -lsgx_tcxx -lsgx_tservice -lsgx_tcrypto \
    -L${MESATEE_OUT_DIR} -lpycomponent ffi.o -lpypy-c -lsgx_tlibc_ext -lffi \
    -L${TRUSTED_TARGET_DIR}/${TARGET} -l${CUR_MODULE_NAME}_enclave -Wl,--end-group \
    -Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined \
    -Wl,-pie,-eenclave_entry -Wl,--export-dynamic  \
    -Wl,--defsym,__ImageBase=0 \
    -Wl,--gc-sections \
    -Wl,--version-script=${TOOLCHAIN_DEPS_DIR}/Enclave.lds
${SGX_ENCLAVE_SIGNER} sign -key ${TOOLCHAIN_DEPS_DIR}/Enclave_private.pem \
    -enclave ${CUR_MODULE_NAME}.enclave.so \
    -out ${MESATEE_SERVICE_INSTALL_DIR}/${CUR_MODULE_NAME}.enclave.signed.so \
    -config ${MESATEE_PROJECT_ROOT}/${CUR_MODULE_PATH}/sgx_trusted_lib/Enclave.config.xml \
    -dumpfile ${CUR_MODULE_NAME}.enclave.meta.txt > /dev/null 2>&1
echo ${CUR_MODULE_NAME} > ${CUR_MODULE_NAME}_enclave_info.txt
grep -m1 -A2 "mrsigner->value" ${CUR_MODULE_NAME}.enclave.meta.txt >> ${CUR_MODULE_NAME}_enclave_info.txt
grep -m1 -A2 "body.enclave_hash" ${CUR_MODULE_NAME}.enclave.meta.txt >> ${CUR_MODULE_NAME}_enclave_info.txt
