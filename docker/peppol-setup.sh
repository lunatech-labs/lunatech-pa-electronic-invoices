#!/bin/bash
# =============================================================================
# Setup PEPPOL local : enregistre les participants dans le SMP local
# =============================================================================
#
# Ce script configure le SMP local (phoss-smp) pour que les 2 instances
# Oxalis puissent s'échanger des messages AS4 en local.
#
# Usage :
#   podman compose --profile peppol up -d
#   ./docker/peppol-setup.sh
#
# Participants enregistrés :
#   - PDP_A (vendeur)  : 0002::123456789  → oxalis:8080
#   - PDP_B (acheteur) : 0002::987654321  → oxalis-remote:8080

set -euo pipefail

SMP_URL="${SMP_URL:-http://localhost:8888}"
SMP_USER="${SMP_USER:-admin@helger.com}"
SMP_PASS="${SMP_PASS:-password}"

# Certificat AP (base64 DER, extrait du keystore de test)
AP_CERT=$(keytool -exportcert -alias ap \
  -keystore "$(dirname "$0")/oxalis/oxalis-keystore.jks" \
  -storepass changeit 2>/dev/null | base64 | tr -d '\n')

echo "=== Setup PEPPOL local ==="
echo "SMP: $SMP_URL"
echo "Cert: ${AP_CERT:0:40}..."

# --- Attendre que le SMP soit prêt ---
echo -n "Attente du SMP..."
for i in $(seq 1 30); do
  if curl -fsSL --max-time 5 "$SMP_URL/public" >/dev/null 2>&1; then
    echo " OK"
    break
  fi
  echo -n "."
  sleep 2
done

# URL-encode un identifiant SMP (:: → %3A%3A, ## → %23%23, etc.)
urlencode_smp() {
  echo -n "$1" | sed 's/:/%3A/g; s/#/%23/g'
}

# --- Participant PDP_B (acheteur, destinataire sur oxalis-remote) ---
PARTICIPANT_B="iso6523-actorid-upis::0002:987654321"
PARTICIPANT_B_ENC=$(urlencode_smp "$PARTICIPANT_B")
DOC_TYPE_UBL="busdox-docid-qns::urn:oasis:names:specification:ubl:schema:xsd:Invoice-2::Invoice##urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0::2.1"
DOC_TYPE_UBL_ENC=$(urlencode_smp "$DOC_TYPE_UBL")
PROCESS_ID="urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"

# L'endpoint AS4 de oxalis-remote (depuis le réseau Docker)
ENDPOINT_B="http://oxalis-remote:8080/as4"

echo ""
echo "--- Enregistrement PDP_B (0002::987654321) ---"

TMPDIR_SMP=$(mktemp -d)
trap "rm -rf $TMPDIR_SMP" EXIT

# 1. Créer le Service Group
echo "  1/2 Service Group..."
cat > "$TMPDIR_SMP/servicegroup.xml" <<'XMLEOF'
<?xml version="1.0" encoding="UTF-8"?>
<smp:ServiceGroup xmlns:smp="http://busdox.org/serviceMetadata/publishing/1.0/" xmlns:id="http://busdox.org/transport/identifiers/1.0/">
  <id:ParticipantIdentifier scheme="iso6523-actorid-upis">0002:987654321</id:ParticipantIdentifier>
  <smp:ServiceMetadataReferenceCollection/>
</smp:ServiceGroup>
XMLEOF
curl -s --max-time 10 -X PUT \
  -u "$SMP_USER:$SMP_PASS" \
  -H "Content-Type: application/xml" \
  -d @"$TMPDIR_SMP/servicegroup.xml" \
  "$SMP_URL/$PARTICIPANT_B_ENC?create-in-sml=false" && echo " OK" || echo " ERREUR"

# 2. Créer le Service Metadata (endpoint AS4)
echo "  2/2 Service Metadata (UBL Invoice → oxalis-remote)..."
cat > "$TMPDIR_SMP/servicemetadata.xml" <<XMLEOF
<?xml version="1.0" encoding="UTF-8"?>
<smp:ServiceMetadata xmlns:smp="http://busdox.org/serviceMetadata/publishing/1.0/" xmlns:id="http://busdox.org/transport/identifiers/1.0/" xmlns:wsa="http://www.w3.org/2005/08/addressing">
  <smp:ServiceInformation>
    <id:ParticipantIdentifier scheme="iso6523-actorid-upis">0002:987654321</id:ParticipantIdentifier>
    <id:DocumentIdentifier scheme="busdox-docid-qns">urn:oasis:names:specification:ubl:schema:xsd:Invoice-2::Invoice##urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0::2.1</id:DocumentIdentifier>
    <smp:ProcessList>
      <smp:Process>
        <id:ProcessIdentifier scheme="cenbii-procid-ubl">$PROCESS_ID</id:ProcessIdentifier>
        <smp:ServiceEndpointList>
          <smp:Endpoint transportProfile="peppol-transport-as4-v2_0">
            <wsa:EndpointReference>
              <wsa:Address>$ENDPOINT_B</wsa:Address>
            </wsa:EndpointReference>
            <smp:RequireBusinessLevelSignature>false</smp:RequireBusinessLevelSignature>
            <smp:Certificate>$AP_CERT</smp:Certificate>
            <smp:ServiceDescription>PDP_B - Access Point destinataire (test local)</smp:ServiceDescription>
            <smp:TechnicalContactUrl>https://github.com/pdp-facture</smp:TechnicalContactUrl>
          </smp:Endpoint>
        </smp:ServiceEndpointList>
      </smp:Process>
    </smp:ProcessList>
  </smp:ServiceInformation>
</smp:ServiceMetadata>
XMLEOF
curl -s --max-time 10 -X PUT \
  -u "$SMP_USER:$SMP_PASS" \
  -H "Content-Type: application/xml" \
  -d @"$TMPDIR_SMP/servicemetadata.xml" \
  "$SMP_URL/$PARTICIPANT_B_ENC/services/$DOC_TYPE_UBL_ENC" && echo " OK" || echo " ERREUR"

# --- Vérification ---
echo ""
echo "--- Vérification ---"
echo -n "  Lookup PDP_B: "
HTTP_CODE=$(curl -s --max-time 10 -o /dev/null -w "%{http_code}" "$SMP_URL/$PARTICIPANT_B_ENC")
if [ "$HTTP_CODE" = "200" ]; then
  echo "OK (HTTP $HTTP_CODE)"
else
  echo "ERREUR (HTTP $HTTP_CODE)"
fi

echo -n "  Lookup PDP_B UBL Invoice: "
RESP_FILE="$TMPDIR_SMP/lookup_response.txt"
HTTP_CODE=$(curl -s --max-time 10 -o "$RESP_FILE" -w "%{http_code}" "$SMP_URL/$PARTICIPANT_B_ENC/services/$DOC_TYPE_UBL_ENC")
if [ "$HTTP_CODE" = "200" ]; then
  echo "OK (HTTP $HTTP_CODE)"
else
  echo "ERREUR (HTTP $HTTP_CODE)"
  echo "  Détail: $(head -c 300 "$RESP_FILE" 2>/dev/null)"
fi

echo ""
echo "=== Setup terminé ==="
echo "PDP_A (oxalis:8080)        → envoie vers PDP_B"
echo "PDP_B (oxalis-remote:8080) → reçoit dans /oxalis/inbound"
echo ""
echo "Test d'envoi :"
echo "  OXALIS_URL=http://localhost:8080 cargo test -p pdp-peppol --test peppol_integration"
