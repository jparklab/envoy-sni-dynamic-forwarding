#!/bin/bash
set -e

NAME=fake-service.mydomain.com

if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <output directory> [<name>]"
    exit 1
fi

OUTDIR=$1

if [ "$#" -gt 1 ]; then 
    NAME=$2
fi

if [ ! -e ${OUTDIR} ]; then
    echo "Output directory '${OUTDIR}' does not exist"
    exit 1
fi
echo "Generating self signed CA certificate/key"
openssl genrsa -out ${OUTDIR}/ca-key.pem 2048
openssl req -new -x509 -days 10 -key ${OUTDIR}/ca-key.pem -sha256 -out ${OUTDIR}/ca-cert.pem \
    -subj "/C=US/ST=New York/L=New York/O=My Company/OU=IT/CN=My Root CA"

echo "Generating server certificate for ${NAME}"

cat > ${OUTDIR}/server-crt.conf << EOF
[ req ]
encrypt_key = no
prompt = no

default_keyfile = ${OUTDIR}/server-key.pem
distinguished_name = name
req_extensions = name_extension

[ name ]
CN = ${NAME}

[ name_extension ]
basicConstraints = CA:FALSE
subjectKeyIdentifier = hash
subjectAltName = @server_alt_names

[ server_alt_names ]
DNS.1 = ${NAME}

EOF

openssl req -new -out ${OUTDIR}/server.csr --config ${OUTDIR}/server-crt.conf
openssl x509 -req -in ${OUTDIR}/server.csr -CA ${OUTDIR}/ca-cert.pem -CAkey ${OUTDIR}/ca-key.pem \
    -CAcreateserial -out ${OUTDIR}/server-cert.pem -days 5 -sha256 \
    -extfile ${OUTDIR}/server-crt.conf