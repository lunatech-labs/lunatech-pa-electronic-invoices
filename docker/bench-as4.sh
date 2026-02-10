#!/bin/bash
# =============================================================================
# Benchmark AS4 : envois séquentiels et parallèles via oxalis-standalone
# =============================================================================
#
# Mesure les performances d'envoi AS4 signé PDP_A → PDP_B :
#   - Séquentiel : N envois l'un après l'autre
#   - Parallèle  : N envois simultanés (via --repeat d'oxalis-standalone)
#
# Prérequis :
#   podman compose --profile peppol up -d smp oxalis oxalis-remote
#   bash ./docker/peppol-setup.sh
#   bash ./docker/bench-as4.sh [N]
#
# Argument optionnel : nombre de messages (défaut: 10)

set -euo pipefail

# Timestamp en millisecondes (portable macOS/Linux)
ms_now() { python3 -c "import time; print(int(time.time()*1000))"; }

N="${1:-10}"
OXALIS_CONTAINER="${OXALIS_CONTAINER:-pdp-oxalis}"
REMOTE_CONTAINER="${REMOTE_CONTAINER:-pdp-oxalis-remote}"

echo "╔══════════════════════════════════════════════════╗"
echo "║        Benchmark AS4 — Oxalis standalone         ║"
echo "╠══════════════════════════════════════════════════╣"
echo "║  Messages : $N"
echo "║  Sender   : 0002:123456789 (oxalis)"
echo "║  Receiver : 0002:987654321 (oxalis-remote)"
echo "╚══════════════════════════════════════════════════╝"
echo ""

# --- Préparation ---
echo "--- Préparation ---"

# Export certificat + copie keystore (si pas déjà fait)
podman exec "$OXALIS_CONTAINER" keytool -exportcert -alias ap \
  -keystore /oxalis/conf/conf/oxalis-keystore.jks \
  -storepass changeit -rfc -file /tmp/ap-cert.pem 2>/dev/null
podman exec "$OXALIS_CONTAINER" cp /oxalis/conf/conf/oxalis-keystore.jks \
  /oxalis/conf/oxalis-keystore.jks 2>/dev/null || true

# Vider l'inbound de PDP_B
podman exec "$REMOTE_CONTAINER" sh -c "rm -rf /oxalis/inbound/*" 2>/dev/null || true

# Générer N fichiers SBDH distincts (chacun avec un UUID unique)
INVOICE_FILE="tests/fixtures/ubl/facture_ubl_001.xml"
INVOICE_CONTENT=$(sed '1{/^<?xml/d;}' "$INVOICE_FILE")

TMPDIR_BENCH=$(mktemp -d)
trap "rm -rf $TMPDIR_BENCH" EXIT

for i in $(seq 1 "$N"); do
  cat > "$TMPDIR_BENCH/sbdh_${i}.xml" <<SBDHEOF
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
done

# Copier tous les SBDH dans le container
for i in $(seq 1 "$N"); do
  podman cp "$TMPDIR_BENCH/sbdh_${i}.xml" "$OXALIS_CONTAINER:/tmp/sbdh_${i}.xml"
done
echo "  $N fichiers SBDH générés et copiés"
echo ""

# --- Commande oxalis-standalone commune ---
OXALIS_CMD="java \
  -DOXALIS_HOME=/oxalis/conf \
  -Dlookup.locator.class=network.oxalis.vefa.peppol.lookup.locator.StaticLocator \
  -Dlookup.locator.hostname=smp \
  -Dlookup.locator.uri=http://smp:8080 \
  -Dsecurity.truststore.ap=/oxalis/conf/oxalis-keystore.jks \
  -Dsecurity.truststore.password=changeit \
  -classpath /oxalis/lib/*:/oxalis/lib-standalone/*:/oxalis/ext/* \
  eu.sendregning.oxalis.Main \
  -r 0002:987654321 \
  -s 0002:123456789 \
  -u http://oxalis-remote:8080/as4 \
  --protocol peppol-transport-as4-v2_0 \
  --cert /tmp/ap-cert.pem \
  -d urn:oasis:names:specification:ubl:schema:xsd:Invoice-2::Invoice##urn:cen.eu:en16931:2017#compliant#urn:fdc:peppol.eu:2017:poacc:billing:3.0::2.1 \
  -p urn:fdc:peppol.eu:2017:poacc:billing:01:1.0"

# ============================================================
# Test 1 : Envois séquentiels (1 par 1)
# ============================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 1 : $N envois SÉQUENTIELS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Vider l'inbound
podman exec "$REMOTE_CONTAINER" sh -c "rm -rf /oxalis/inbound/*" 2>/dev/null || true

SEQ_START=$(ms_now)

SEQ_OK=0
SEQ_FAIL=0
for i in $(seq 1 "$N"); do
  RESULT=$(podman exec "$OXALIS_CONTAINER" sh -c "$OXALIS_CMD -f /tmp/sbdh_${i}.xml 2>&1" | grep -c "Failed transmissions: 0" || true)
  if [ "$RESULT" -ge 1 ]; then
    SEQ_OK=$((SEQ_OK + 1))
  else
    SEQ_FAIL=$((SEQ_FAIL + 1))
  fi
  echo -n "."
done
echo ""

SEQ_END=$(ms_now)
SEQ_DURATION=$((SEQ_END - SEQ_START))
SEQ_AVG=$((SEQ_DURATION / N))
SEQ_THROUGHPUT=$(echo "scale=1; $N * 1000 / $SEQ_DURATION" | bc 2>/dev/null || echo "N/A")

SEQ_RECEIVED=$(podman exec "$REMOTE_CONTAINER" find /oxalis/inbound -type f 2>/dev/null | wc -l | tr -d ' ')

echo ""
echo "  Résultats séquentiels :"
echo "  ├─ Envoyés   : $N ($SEQ_OK OK, $SEQ_FAIL échecs)"
echo "  ├─ Reçus     : $SEQ_RECEIVED fichiers dans /oxalis/inbound"
echo "  ├─ Durée     : ${SEQ_DURATION}ms"
echo "  ├─ Moyenne   : ${SEQ_AVG}ms/message"
echo "  └─ Débit     : ${SEQ_THROUGHPUT} msg/s"
echo ""

# ============================================================
# Test 2 : Envois parallèles (--repeat N sur un seul fichier)
# ============================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 2 : $N envois PARALLÈLES (--repeat)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Vider l'inbound
podman exec "$REMOTE_CONTAINER" sh -c "rm -rf /oxalis/inbound/*" 2>/dev/null || true

PAR_START=$(ms_now)

PAR_OUTPUT=$(podman exec "$OXALIS_CONTAINER" sh -c "$OXALIS_CMD -f /tmp/sbdh_1.xml --repeat $N 2>&1")

PAR_END=$(ms_now)
PAR_DURATION=$((PAR_END - PAR_START))

# Extraire les stats d'oxalis-standalone
PAR_AVG=$(echo "$PAR_OUTPUT" | grep "Average transmission" | grep -oE '[0-9]+(\.[0-9]+)?' | head -1 || echo "N/A")
PAR_SPEED=$(echo "$PAR_OUTPUT" | grep "Transmission speed" | grep -oE '[0-9]+(\.[0-9]+)?' | head -1 || echo "N/A")
PAR_FAILED=$(echo "$PAR_OUTPUT" | grep "Failed transmissions" | grep -oE '[0-9]+' || echo "?")
PAR_TOTAL_TIME=$(echo "$PAR_OUTPUT" | grep "Total time spent" || echo "N/A")

PAR_RECEIVED=$(podman exec "$REMOTE_CONTAINER" find /oxalis/inbound -type f 2>/dev/null | wc -l | tr -d ' ')

echo ""
echo "  Résultats parallèles (oxalis --repeat $N) :"
echo "  ├─ $PAR_TOTAL_TIME"
echo "  ├─ Échecs    : $PAR_FAILED"
echo "  ├─ Reçus     : $PAR_RECEIVED fichiers dans /oxalis/inbound"
echo "  ├─ Durée     : ${PAR_DURATION}ms (wall clock)"
echo "  ├─ Moyenne   : ${PAR_AVG}ms/message (oxalis interne)"
echo "  └─ Débit     : ${PAR_SPEED} msg/s (oxalis interne)"
echo ""

# ============================================================
# Test 3 : Envois parallèles (N processus shell concurrents)
# ============================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 3 : $N envois CONCURRENTS (shell background)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Vider l'inbound
podman exec "$REMOTE_CONTAINER" sh -c "rm -rf /oxalis/inbound/*" 2>/dev/null || true

CON_START=$(ms_now)

# Lancer N envois en parallèle
PIDS=""
for i in $(seq 1 "$N"); do
  podman exec "$OXALIS_CONTAINER" sh -c "$OXALIS_CMD -f /tmp/sbdh_${i}.xml 2>&1" > "$TMPDIR_BENCH/result_${i}.log" 2>&1 &
  PIDS="$PIDS $!"
done

# Attendre tous les processus
CON_OK=0
CON_FAIL=0
for pid in $PIDS; do
  wait "$pid" 2>/dev/null || true
done

CON_END=$(ms_now)
CON_DURATION=$((CON_END - CON_START))

# Compter les succès
for i in $(seq 1 "$N"); do
  if grep -q "Failed transmissions: 0" "$TMPDIR_BENCH/result_${i}.log" 2>/dev/null; then
    CON_OK=$((CON_OK + 1))
  else
    CON_FAIL=$((CON_FAIL + 1))
  fi
done

CON_AVG=$((CON_DURATION / N))
CON_THROUGHPUT=$(echo "scale=1; $N * 1000 / $CON_DURATION" | bc 2>/dev/null || echo "N/A")
CON_RECEIVED=$(podman exec "$REMOTE_CONTAINER" find /oxalis/inbound -type f 2>/dev/null | wc -l | tr -d ' ')

echo ""
echo "  Résultats concurrents ($N processus shell) :"
echo "  ├─ Envoyés   : $N ($CON_OK OK, $CON_FAIL échecs)"
echo "  ├─ Reçus     : $CON_RECEIVED fichiers dans /oxalis/inbound"
echo "  ├─ Durée     : ${CON_DURATION}ms (wall clock)"
echo "  ├─ Moyenne   : ${CON_AVG}ms/message (wall clock / N)"
echo "  └─ Débit     : ${CON_THROUGHPUT} msg/s"
echo ""

# ============================================================
# Résumé
# ============================================================
echo "╔══════════════════════════════════════════════════╗"
echo "║              RÉSUMÉ BENCHMARK AS4               ║"
echo "╠══════════════════════════════════════════════════╣"
printf "║  %-12s │ %8s │ %8s │ %7s ║\n" "Mode" "Durée" "Moy/msg" "Débit"
echo "║──────────────┼──────────┼──────────┼─────────║"
printf "║  %-12s │ %6dms │ %6dms │ %5s/s ║\n" "Séquentiel" "$SEQ_DURATION" "$SEQ_AVG" "$SEQ_THROUGHPUT"
printf "║  %-12s │ %6dms │ %6sms │ %5s/s ║\n" "Parallèle" "$PAR_DURATION" "$PAR_AVG" "$PAR_SPEED"
printf "║  %-12s │ %6dms │ %6dms │ %5s/s ║\n" "Concurrent" "$CON_DURATION" "$CON_AVG" "$CON_THROUGHPUT"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "  Facture : $(wc -c < "$INVOICE_FILE" | tr -d ' ') octets (facture_ubl_001.xml)"
echo "  SBDH    : $(wc -c < "$TMPDIR_BENCH/sbdh_1.xml" | tr -d ' ') octets (avec enveloppe)"
