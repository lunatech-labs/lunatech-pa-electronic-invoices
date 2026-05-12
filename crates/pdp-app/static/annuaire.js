    const searchInput = document.getElementById('searchInput');
    const resultsDiv = document.getElementById('results');
    const spinner = document.getElementById('spinner');
    const resultCount = document.getElementById('resultCount');
    const searchTime = document.getElementById('searchTime');

    let debounceTimer = null;
    let currentQuery = '';

    searchInput.addEventListener('input', () => {
        clearTimeout(debounceTimer);
        const q = searchInput.value.trim();
        if (q.length < 3) {
            resultsDiv.innerHTML = `<div class="empty-state">
                <div class="icon">&#128218;</div>
                <p>Saisissez au moins 3 caracteres</p>
            </div>`;
            resultCount.textContent = '';
            searchTime.textContent = '';
            return;
        }
        debounceTimer = setTimeout(() => doSearch(q), 250);
    });

    // Pre-remplit la recherche depuis ?q=... pour les liens partageables
    // et les screenshots de doc.
    (() => {
        const params = new URLSearchParams(window.location.search);
        const q0 = (params.get('q') || '').trim();
        if (q0.length >= 3) {
            searchInput.value = q0;
            doSearch(q0);
        }
    })();

    async function doSearch(query) {
        if (query === currentQuery) return;
        currentQuery = query;
        spinner.classList.add('visible');
        resultCount.textContent = 'Recherche...';
        searchTime.textContent = '';

        const start = performance.now();
        try {
            const res = await fetch(`/v1/annuaire/search?q=${encodeURIComponent(query)}`);
            const data = await res.json();
            const elapsed = ((performance.now() - start) / 1000).toFixed(2);

            if (query !== currentQuery) return; // stale

            spinner.classList.remove('visible');

            if (!data.results || data.results.length === 0) {
                resultsDiv.innerHTML = `<div class="empty-state">
                    <div class="icon">&#128533;</div>
                    <p>Aucun resultat pour "${escHtml(query)}"</p>
                </div>`;
                resultCount.textContent = '0 resultat';
                searchTime.textContent = `${elapsed}s`;
                return;
            }

            // Grouper par SIREN
            const grouped = groupBySiren(data.results);
            resultCount.textContent = `${Object.keys(grouped).length} entreprise(s)`;
            searchTime.textContent = `${elapsed}s`;

            resultsDiv.innerHTML = Object.values(grouped).map(renderCompany).join('');
        } catch (e) {
            spinner.classList.remove('visible');
            resultsDiv.innerHTML = `<div class="empty-state">
                <div class="icon">&#9888;&#65039;</div>
                <p>Erreur : ${escHtml(e.message)}</p>
            </div>`;
        }
    }

    function groupBySiren(results) {
        const groups = {};
        for (const r of results) {
            const siren = r.siren.trim();
            if (!groups[siren]) {
                groups[siren] = {
                    siren,
                    nom: r.nom.trim(),
                    type_entite: r.type_entite.trim(),
                    statut: r.statut.trim(),
                    diffusible: r.diffusible,
                    plateforme: r.plateforme ? r.plateforme.trim() : null,
                    plateforme_nom: r.plateforme_nom ? r.plateforme_nom.trim() : null,
                    plateforme_type: r.plateforme_type ? r.plateforme_type.trim() : null,
                    plateforme_nom_commercial: r.plateforme_nom_commercial ? r.plateforme_nom_commercial.trim() : null,
                    la_date_debut: r.la_date_debut || null,
                    la_date_fin: r.la_date_fin || null,
                    etablissements: [],
                };
            }
            if (r.siret) {
                const siret = r.siret.trim();
                if (!groups[siren].etablissements.find(e => e.siret === siret)) {
                    groups[siren].etablissements.push({
                        siret,
                        nom: r.etab_nom ? r.etab_nom.trim() : '',
                        type_etablissement: r.type_etablissement ? r.type_etablissement.trim() : '',
                        adresse_1: r.adresse_1 ? r.adresse_1.trim() : '',
                        adresse_2: r.adresse_2 ? r.adresse_2.trim() : '',
                        adresse_3: r.adresse_3 ? r.adresse_3.trim() : '',
                        localite: r.localite ? r.localite.trim() : '',
                        cp: r.code_postal ? r.code_postal.trim() : '',
                        code_pays: r.code_pays ? r.code_pays.trim() : 'FR',
                        statut: r.etab_statut ? r.etab_statut.trim() : '',
                        engagement_juridique: r.engagement_juridique,
                        service: r.service,
                        moa: r.moa,
                        id_routage: r.id_routage ? r.id_routage.trim() : null,
                    });
                }
            }
        }
        return groups;
    }

    function renderCompany(c) {
        const typeLabel = c.type_entite === 'A' ? 'Assujetti' : 'Personne physique';
        const statusLabel = c.statut === 'A' ? 'Actif' : 'Inactif';
        const statusClass = c.statut === 'A' ? '' : ' inactif';
        const diffLabel = c.diffusible === true ? 'Oui' : c.diffusible === false ? 'Non' : '—';

        let pdpBadge = '';
        if (c.plateforme) {
            const pdpLabel = c.plateforme === '9998' ? 'PPF (fictive)' :
                             c.plateforme === '0000' ? 'PPF' :
                             c.plateforme === '9999' ? 'Chorus Pro' :
                             `PDP ${c.plateforme}${c.plateforme_nom ? ' - ' + escHtml(c.plateforme_nom) : ''}`;
            pdpBadge = `<span class="result-badge badge-pdp">${pdpLabel}</span>`;
        }

        const typeEtabLabel = (code) => {
            if (code === 'S') return 'Siege';
            if (code === 'P') return 'Principal';
            if (code === 'E') return 'Secondaire';
            return code || '—';
        };

        let etabHtml = '';
        if (c.etablissements.length > 0) {
            const rows = c.etablissements.map(e => {
                const adresseLines = [e.adresse_1, e.adresse_2, e.adresse_3].filter(Boolean);
                const adresseStr = adresseLines.join(', ');
                const lieuStr = [e.cp, e.localite].filter(Boolean).join(' ');
                const paysStr = e.code_pays && e.code_pays !== 'FR' ? ` (${escHtml(e.code_pays)})` : '';
                const etabStatus = e.statut === 'I' ? '<span class="result-badge badge-status inactif" style="margin-left:0.3rem">Inactif</span>' : '';
                const b2gBadges = [
                    e.engagement_juridique ? '<span class="result-badge badge-b2g">EJ</span>' : '',
                    e.service ? '<span class="result-badge badge-b2g">Service</span>' : '',
                    e.moa ? '<span class="result-badge badge-b2g">MOA</span>' : '',
                ].filter(Boolean).join('');
                const routageBadge = e.id_routage ? `<span class="result-badge badge-routage">${escHtml(e.id_routage)}</span>` : '';

                return `
                <div class="etab-row" style="flex-direction:column;gap:0.2rem">
                    <div style="display:flex;justify-content:space-between;align-items:center">
                        <div>
                            <span class="etab-siret">${escHtml(e.siret)}</span>
                            <span class="result-badge badge-type" style="margin-left:0.3rem">${typeEtabLabel(e.type_etablissement)}</span>
                            ${etabStatus}
                            ${e.nom ? ' — ' + escHtml(e.nom) : ''}
                        </div>
                        <div>${b2gBadges}${routageBadge}</div>
                    </div>
                    <div style="color:#888;font-size:0.8rem">
                        ${adresseStr ? escHtml(adresseStr) + '<br>' : ''}${escHtml(lieuStr)}${paysStr}
                    </div>
                </div>`;
            }).join('');
            etabHtml = `
                <div class="etab-list">
                    <h4>Etablissements (${c.etablissements.length})</h4>
                    ${rows}
                </div>
            `;
        }

        return `
        <div class="result-card" onclick="this.classList.toggle('expanded')">
            <div class="result-header">
                <div>
                    <div class="result-name">${escHtml(c.nom)}</div>
                    <div class="result-meta">
                        <span class="result-badge badge-type">${typeLabel}</span>
                        <span class="result-badge badge-status${statusClass}">${statusLabel}</span>
                        ${pdpBadge}
                    </div>
                </div>
                <span class="result-siren">${escHtml(c.siren)}</span>
            </div>
            <div class="detail-section">
                <div class="detail-grid">
                    <div class="detail-item">
                        <div class="detail-label">SIREN</div>
                        <div class="detail-value">${escHtml(c.siren)}</div>
                    </div>
                    <div class="detail-item">
                        <div class="detail-label">Plateforme</div>
                        <div class="detail-value">${c.plateforme ? escHtml(c.plateforme) + (c.plateforme_nom_commercial || c.plateforme_nom ? ' — ' + escHtml(c.plateforme_nom_commercial || c.plateforme_nom) : '') + (c.plateforme_type ? ' (' + escHtml(c.plateforme_type) + ')' : '') : 'Non renseigne'}</div>
                    </div>
                    <div class="detail-item">
                        <div class="detail-label">Type</div>
                        <div class="detail-value">${typeLabel}</div>
                    </div>
                    <div class="detail-item">
                        <div class="detail-label">Statut</div>
                        <div class="detail-value">${statusLabel}</div>
                    </div>
                    <div class="detail-item">
                        <div class="detail-label">Diffusible</div>
                        <div class="detail-value">${diffLabel}</div>
                    </div>
                    <div class="detail-item">
                        <div class="detail-label">Inscription annuaire</div>
                        <div class="detail-value">${c.la_date_debut ? escHtml(c.la_date_debut) + (c.la_date_fin ? ' → ' + escHtml(c.la_date_fin) : ' → en cours') : '—'}</div>
                    </div>
                </div>
                ${etabHtml}
            </div>
        </div>
        `;
    }

    function escHtml(s) {
        if (!s) return '';
        return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
    }
