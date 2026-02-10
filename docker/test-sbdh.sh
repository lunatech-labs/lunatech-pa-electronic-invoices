#!/bin/bash
# =============================================================================
# Test d'envoi AS4 réel via oxalis-standalone
# =============================================================================
#
# Ce script :
# 1. Génère un SBDH wrappant une facture UBL de test
# 2. L'envoie via oxalis-standalone (PDP_A → PDP_B)
# 3. Vérifie que le message arrive dans /oxalis/inbound de PDP_B
#
# Prérequis :
#   podman compose --profile peppol up -d smp oxalis oxalis-remote
#   bash ./docker/peppol-setup.sh
#   bash ./docker/test-sbdh.sh

set -euo pipefail

OXALIS_CONTAINER="${OXALIS_CONTAINER:-pdp-oxalis}"
REMOTE_CONTAINER="${REMOTE_CONTAINER:-pdp-oxalis-remote}"

echo "=== Test envoi AS4 réel (oxalis-standalone) ==="

# --- 1. Préparer le certificat destinataire ---
echo "  1/4 Export certificat AP..."
podman exec "$OXALIS_CONTAINER" keytool -exportcert -alias ap \
  -keystore /oxalis/conf/conf/oxalis-keystore.jks \
  -storepass changeit -rfc -file /tmp/ap-cert.pem 2>/dev/null
echo " OK"

# --- 2. Copier le keystore au bon endroit pour standalone ---
echo "  2/4 Copie keystore..."
podman exec "$OXALIS_CONTAINER" cp /oxalis/conf/conf/oxalis-keystore.jks /oxalis/conf/oxalis-keystore.jks 2>/dev/null || true
echo " OK"

# --- 3. Générer le SBDH avec la facture UBL embarquée ---
echo "  3/4 Génération SBDH..."

# Lire la facture UBL et échapper pour inclusion dans le SBDH
INVOICE_FILE="tests/fixtures/ubl/facture_ubl_001.xml"
if [ ! -f "$INVOICE_FILE" ]; then
  echo "ERREUR: $INVOICE_FILE introuvable"
  exit 1
fi

# Créer le SBDH (sans la déclaration XML du payload)
TMPDIR_TEST=$(mktemp -d)
trap "rm -rf $TMPDIR_TEST" EXIT

# Extraire le contenu UBL sans la déclaration <?xml ...?>
INVOICE_CONTENT=$(sed '1{/^<?xml/d;}' "$INVOICE_FILE")

cat > "$TMPDIR_TEST/sbdh.xml" <<SBDHEOF
<?xml version="1.0" encoding="UTF-8"?>
<StandardBusinessDocument xmlns="http://www.unece.org/cefact/namespaces/StandardBusinessDocumentHeader">
  <StandardBusinessDocumentHeader>
    <HeaderVersion>1.0</HeaderVersion>
    <Sender>
      <Identifier Authority="iso6523-actorid-upis">0002:123456789</Identifier>
    </Sender>
    <Receiver>
      <Identifier Authority="iso6523-actorid-upis">0002:987654321</Identifier>
    </Receiver>
    <DocumentIdentification>
      <Standard>urn:oasis:names:specification:ubl:schema:xsd:Invoice-2</Standard>
      <TypeVersion>2.1</TypeVersion>
      <InstanceIdentifier>$(uuidgen)</InstanceIdentifier>
      <Type>Invoice</Type>
      <CreationDateAndTime>$(date -u +%Y-%m-%dT%H:%M:%S.000Z)</CreationDateAndTime>
    </DocumentIdentification>
    <BusinessScope>
      <Scope>
        <Type>DOCUMENTID</Type>
        <InstanceIdentifier>urn:oasis:names:specification:ubl:schema:xsd:Invoice-2::Invoice##urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0::2.1</InstanceIdentifier>
      </Scope>
      <Scope>
        <Type>PROCESSID</Type>
        <InstanceIdentifier>urn:fdc:peppol.eu:2017:poacc:billing:01:1.0</InstanceIdentifier>
      </Scope>
      <Scope>
        <Type>COUNTRY_C1</Type>
        <InstanceIdentifier>FR</InstanceIdentifier>
      </Scope>
    </BusinessScope>
  </StandardBusinessDocumentHeader>
$INVOICE_CONTENT
</StandardBusinessDocument>
SBDHEOF

# Copier le SBDH dans le container
podman cp "$TMPDIR_TEST/sbdh.xml" "$OXALIS_CONTAINER:/tmp/sbdh.xml"
echo " OK"

# --- 4. Envoyer via oxalis-standalone ---
echo "  4/4 Envoi AS4 via oxalis-standalone..."
podman exec "$OXALIS_CONTAINER" java \
  -DOXALIS_HOME=/oxalis/conf \
  -Dlookup.locator.class=network.oxalis.vefa.peppol.lookup.locator.StaticLocator \
  -Dlookup.locator.hostname=smp \
  -Dlookup.locator.uri=http://smp:8080 \
  -Dsecurity.truststore.ap=/oxalis/conf/oxalis-keystore.jks \
  -Dsecurity.truststore.password=changeit \
  -classpath "/oxalis/lib/*:/oxalis/lib-standalone/*:/oxalis/ext/*" \
  eu.sendregning.oxalis.Main \
  -f /tmp/sbdh.xml \
  -r 0002:987654321 \
  -s 0002:123456789 \
  -u http://oxalis-remote:8080/as4 \
  --protocol peppol-transport-as4-v2_0 \
  --cert /tmp/ap-cert.pem \
  -d "urn:oasis:names:specification:ubl:schema:xsd:Invoice-2::Invoice##urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0::2.1" \
  -p "urn:fdc:peppol.eu:2017:poacc:billing:01:1.0" \
  2>&1

echo ""

# --- 5. Vérifier la réception ---
echo "--- Vérification réception PDP_B ---"
INBOUND_FILES=$(podman exec "$REMOTE_CONTAINER" find /oxalis/inbound -type f 2>/dev/null | wc -l | tr -d ' ')
echo "  Fichiers dans /oxalis/inbound : $INBOUND_FILES"

if [ "$INBOUND_FILES" -gt 0 ]; then
  echo "  ✓ Message AS4 reçu et persisté !"
  podman exec "$REMOTE_CONTAINER" ls -la /oxalis/inbound/
else
  echo "  ✗ Aucun message reçu dans /oxalis/inbound"
fi
