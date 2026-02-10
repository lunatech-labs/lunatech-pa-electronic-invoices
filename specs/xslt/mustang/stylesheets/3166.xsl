<?xml version="1.0"?>
<xsl:stylesheet version="2.0"
	xmlns:xsl="http://www.w3.org/1999/XSL/Transform">

	<xsl:template name="code.Country-Codes">
		<xsl:param name="myparam"/>
		<xsl:variable name="myparam.upper" select="upper-case($myparam)"/>
		<xsl:choose>
      		<xsl:when test="$myparam.upper='AF'"><xsl:value-of select="$myparam"/> (Afghanistan)</xsl:when>
      		<xsl:when test="$myparam.upper='EG'"><xsl:value-of select="$myparam"/> (Égypte)</xsl:when>
      		<xsl:when test="$myparam.upper='AX'"><xsl:value-of select="$myparam"/> (Îles Åland)</xsl:when>
      		<xsl:when test="$myparam.upper='AL'"><xsl:value-of select="$myparam"/> (Albanie)</xsl:when>
      		<xsl:when test="$myparam.upper='DZ'"><xsl:value-of select="$myparam"/> (Algérie)</xsl:when>
      		<xsl:when test="$myparam.upper='VI'"><xsl:value-of select="$myparam"/> (Îles Vierges américaines)</xsl:when>
      		<xsl:when test="$myparam.upper='UM'"><xsl:value-of select="$myparam"/> (Îles mineures éloignées des États-Unis)</xsl:when>
      		<xsl:when test="$myparam.upper='AS'"><xsl:value-of select="$myparam"/> (Samoa américaines)</xsl:when>
      		<xsl:when test="$myparam.upper='AD'"><xsl:value-of select="$myparam"/> (Andorre)</xsl:when>
      		<xsl:when test="$myparam.upper='AO'"><xsl:value-of select="$myparam"/> (Angola)</xsl:when>
      		<xsl:when test="$myparam.upper='AI'"><xsl:value-of select="$myparam"/> (Anguilla)</xsl:when>
      		<xsl:when test="$myparam.upper='AQ'"><xsl:value-of select="$myparam"/> (Antarctique)</xsl:when>
      		<xsl:when test="$myparam.upper='AG'"><xsl:value-of select="$myparam"/> (Antigua-et-Barbuda)</xsl:when>
      		<xsl:when test="$myparam.upper='GQ'"><xsl:value-of select="$myparam"/> (Guinée équatoriale)</xsl:when>
      		<xsl:when test="$myparam.upper='SY'"><xsl:value-of select="$myparam"/> (République arabe syrienne)</xsl:when>
      		<xsl:when test="$myparam.upper='AR'"><xsl:value-of select="$myparam"/> (Argentine)</xsl:when>
      		<xsl:when test="$myparam.upper='AM'"><xsl:value-of select="$myparam"/> (Arménie)</xsl:when>
      		<xsl:when test="$myparam.upper='AW'"><xsl:value-of select="$myparam"/> (Aruba)</xsl:when>
      		<xsl:when test="$myparam.upper='AZ'"><xsl:value-of select="$myparam"/> (Azerbaïdjan)</xsl:when>
      		<xsl:when test="$myparam.upper='ET'"><xsl:value-of select="$myparam"/> (Éthiopie)</xsl:when>
      		<xsl:when test="$myparam.upper='AU'"><xsl:value-of select="$myparam"/> (Australie)</xsl:when>
      		<xsl:when test="$myparam.upper='BS'"><xsl:value-of select="$myparam"/> (Bahamas)</xsl:when>
      		<xsl:when test="$myparam.upper='BH'"><xsl:value-of select="$myparam"/> (Bahreïn)</xsl:when>
      		<xsl:when test="$myparam.upper='BD'"><xsl:value-of select="$myparam"/> (Bangladesh)</xsl:when>
      		<xsl:when test="$myparam.upper='BB'"><xsl:value-of select="$myparam"/> (Barbade)</xsl:when>
      		<xsl:when test="$myparam.upper='BE'"><xsl:value-of select="$myparam"/> (Belgique)</xsl:when>
      		<xsl:when test="$myparam.upper='BZ'"><xsl:value-of select="$myparam"/> (Belize)</xsl:when>
      		<xsl:when test="$myparam.upper='BJ'"><xsl:value-of select="$myparam"/> (Bénin)</xsl:when>
      		<xsl:when test="$myparam.upper='BM'"><xsl:value-of select="$myparam"/> (Bermudes)</xsl:when>
      		<xsl:when test="$myparam.upper='BT'"><xsl:value-of select="$myparam"/> (Bhoutan)</xsl:when>
      		<xsl:when test="$myparam.upper='VE'"><xsl:value-of select="$myparam"/> (Venezuela)</xsl:when>
      		<xsl:when test="$myparam.upper='BQ'"><xsl:value-of select="$myparam"/> (Bonaire, Saint-Eustache et Saba)</xsl:when>
      		<xsl:when test="$myparam.upper='BA'"><xsl:value-of select="$myparam"/> (Bosnie-Herzégovine)</xsl:when>
      		<xsl:when test="$myparam.upper='BW'"><xsl:value-of select="$myparam"/> (Botswana)</xsl:when>
      		<xsl:when test="$myparam.upper='BV'"><xsl:value-of select="$myparam"/> (Île Bouvet)</xsl:when>
      		<xsl:when test="$myparam.upper='BR'"><xsl:value-of select="$myparam"/> (Brésil)</xsl:when>
      		<xsl:when test="$myparam.upper='VG'"><xsl:value-of select="$myparam"/> (Îles Vierges britanniques)</xsl:when>
      		<xsl:when test="$myparam.upper='IO'"><xsl:value-of select="$myparam"/> (Territoire britannique de l'océan Indien)</xsl:when>
      		<xsl:when test="$myparam.upper='BN'"><xsl:value-of select="$myparam"/> (Brunei Darussalam)</xsl:when>
      		<xsl:when test="$myparam.upper='BG'"><xsl:value-of select="$myparam"/> (Bulgarie)</xsl:when>
      		<xsl:when test="$myparam.upper='BF'"><xsl:value-of select="$myparam"/> (Burkina Faso)</xsl:when>
      		<xsl:when test="$myparam.upper='BI'"><xsl:value-of select="$myparam"/> (Burundi)</xsl:when>
      		<xsl:when test="$myparam.upper='CV'"><xsl:value-of select="$myparam"/> (Cabo Verde)</xsl:when>
      		<xsl:when test="$myparam.upper='CL'"><xsl:value-of select="$myparam"/> (Chili)</xsl:when>
      		<xsl:when test="$myparam.upper='CN'"><xsl:value-of select="$myparam"/> (Chine)</xsl:when>
      		<xsl:when test="$myparam.upper='CK'"><xsl:value-of select="$myparam"/> (Îles Cook)</xsl:when>
      		<xsl:when test="$myparam.upper='CR'"><xsl:value-of select="$myparam"/> (Costa Rica)</xsl:when>
      		<xsl:when test="$myparam.upper='CI'"><xsl:value-of select="$myparam"/> (Côte d'Ivoire)</xsl:when>
      		<xsl:when test="$myparam.upper='CW'"><xsl:value-of select="$myparam"/> (Curaçao)</xsl:when>
      		<xsl:when test="$myparam.upper='DK'"><xsl:value-of select="$myparam"/> (Danemark)</xsl:when>
      		<xsl:when test="$myparam.upper='CD'"><xsl:value-of select="$myparam"/> (République démocratique du Congo)</xsl:when>
      		<xsl:when test="$myparam.upper='KP'"><xsl:value-of select="$myparam"/> (République populaire démocratique de Corée)</xsl:when>
      		<xsl:when test="$myparam.upper='LA'"><xsl:value-of select="$myparam"/> (République démocratique populaire lao)</xsl:when>
      		<xsl:when test="$myparam.upper='DE'"><xsl:value-of select="$myparam"/> (Allemagne)</xsl:when>
      		<xsl:when test="$myparam.upper='DM'"><xsl:value-of select="$myparam"/> (Dominique)</xsl:when>
      		<xsl:when test="$myparam.upper='DO'"><xsl:value-of select="$myparam"/> (République dominicaine)</xsl:when>
      		<xsl:when test="$myparam.upper='DJ'"><xsl:value-of select="$myparam"/> (Djibouti)</xsl:when>
      		<xsl:when test="$myparam.upper='EC'"><xsl:value-of select="$myparam"/> (Équateur)</xsl:when>
      		<xsl:when test="$myparam.upper='MK'"><xsl:value-of select="$myparam"/> (Macédoine du Nord)</xsl:when>
      		<xsl:when test="$myparam.upper='SV'"><xsl:value-of select="$myparam"/> (El Salvador)</xsl:when>
      		<xsl:when test="$myparam.upper='ER'"><xsl:value-of select="$myparam"/> (Érythrée)</xsl:when>
      		<xsl:when test="$myparam.upper='EE'"><xsl:value-of select="$myparam"/> (Estonie)</xsl:when>
      		<xsl:when test="$myparam.upper='FK'"><xsl:value-of select="$myparam"/> (Îles Malouines)</xsl:when>
      		<xsl:when test="$myparam.upper='FO'"><xsl:value-of select="$myparam"/> (Îles Féroé)</xsl:when>
      		<xsl:when test="$myparam.upper='FJ'"><xsl:value-of select="$myparam"/> (Fidji)</xsl:when>
      		<xsl:when test="$myparam.upper='FI'"><xsl:value-of select="$myparam"/> (Finlande)</xsl:when>
      		<xsl:when test="$myparam.upper='FM'"><xsl:value-of select="$myparam"/> (États fédérés de Micronésie)</xsl:when>
      		<xsl:when test="$myparam.upper='FR'"><xsl:value-of select="$myparam"/> (France)</xsl:when>
      		<xsl:when test="$myparam.upper='TF'"><xsl:value-of select="$myparam"/> (Terres australes et antarctiques françaises)</xsl:when>
      		<xsl:when test="$myparam.upper='GF'"><xsl:value-of select="$myparam"/> (Guyane française)</xsl:when>
      		<xsl:when test="$myparam.upper='PF'"><xsl:value-of select="$myparam"/> (Polynésie française)</xsl:when>
      		<xsl:when test="$myparam.upper='GA'"><xsl:value-of select="$myparam"/> (Gabon)</xsl:when>
      		<xsl:when test="$myparam.upper='GM'"><xsl:value-of select="$myparam"/> (Gambie)</xsl:when>
      		<xsl:when test="$myparam.upper='GE'"><xsl:value-of select="$myparam"/> (Géorgie)</xsl:when>
      		<xsl:when test="$myparam.upper='GH'"><xsl:value-of select="$myparam"/> (Ghana)</xsl:when>
      		<xsl:when test="$myparam.upper='GI'"><xsl:value-of select="$myparam"/> (Gibraltar)</xsl:when>
      		<xsl:when test="$myparam.upper='GD'"><xsl:value-of select="$myparam"/> (Grenade)</xsl:when>
      		<xsl:when test="$myparam.upper='GR'"><xsl:value-of select="$myparam"/> (Grèce)</xsl:when>
      		<xsl:when test="$myparam.upper='GL'"><xsl:value-of select="$myparam"/> (Groenland)</xsl:when>
      		<xsl:when test="$myparam.upper='GP'"><xsl:value-of select="$myparam"/> (Guadeloupe)</xsl:when>
      		<xsl:when test="$myparam.upper='GU'"><xsl:value-of select="$myparam"/> (Guam)</xsl:when>
      		<xsl:when test="$myparam.upper='GT'"><xsl:value-of select="$myparam"/> (Guatemala)</xsl:when>
      		<xsl:when test="$myparam.upper='GG'"><xsl:value-of select="$myparam"/> (Guernesey)</xsl:when>
      		<xsl:when test="$myparam.upper='GN'"><xsl:value-of select="$myparam"/> (Guinée)</xsl:when>
      		<xsl:when test="$myparam.upper='GW'"><xsl:value-of select="$myparam"/> (Guinée-Bissau)</xsl:when>
      		<xsl:when test="$myparam.upper='GY'"><xsl:value-of select="$myparam"/> (Guyana)</xsl:when>
      		<xsl:when test="$myparam.upper='HT'"><xsl:value-of select="$myparam"/> (Haïti)</xsl:when>
      		<xsl:when test="$myparam.upper='HM'"><xsl:value-of select="$myparam"/> (Îles Heard-et-MacDonald)</xsl:when>
      		<xsl:when test="$myparam.upper='HN'"><xsl:value-of select="$myparam"/> (Honduras)</xsl:when>
      		<xsl:when test="$myparam.upper='HK'"><xsl:value-of select="$myparam"/> (Hong Kong)</xsl:when>
      		<xsl:when test="$myparam.upper='IN'"><xsl:value-of select="$myparam"/> (Inde)</xsl:when>
      		<xsl:when test="$myparam.upper='ID'"><xsl:value-of select="$myparam"/> (Indonésie)</xsl:when>
      		<xsl:when test="$myparam.upper='IM'"><xsl:value-of select="$myparam"/> (Île de Man)</xsl:when>
      		<xsl:when test="$myparam.upper='IQ'"><xsl:value-of select="$myparam"/> (Irak)</xsl:when>
      		<xsl:when test="$myparam.upper='IE'"><xsl:value-of select="$myparam"/> (Irlande)</xsl:when>
      		<xsl:when test="$myparam.upper='IR'"><xsl:value-of select="$myparam"/> (République islamique d'Iran)</xsl:when>
      		<xsl:when test="$myparam.upper='IS'"><xsl:value-of select="$myparam"/> (Islande)</xsl:when>
      		<xsl:when test="$myparam.upper='IL'"><xsl:value-of select="$myparam"/> (Israël)</xsl:when>
      		<xsl:when test="$myparam.upper='IT'"><xsl:value-of select="$myparam"/> (Italie)</xsl:when>
      		<xsl:when test="$myparam.upper='JM'"><xsl:value-of select="$myparam"/> (Jamaïque)</xsl:when>
      		<xsl:when test="$myparam.upper='JP'"><xsl:value-of select="$myparam"/> (Japon)</xsl:when>
      		<xsl:when test="$myparam.upper='YE'"><xsl:value-of select="$myparam"/> (Yémen)</xsl:when>
      		<xsl:when test="$myparam.upper='JE'"><xsl:value-of select="$myparam"/> (Jersey)</xsl:when>
      		<xsl:when test="$myparam.upper='JO'"><xsl:value-of select="$myparam"/> (Jordanie)</xsl:when>
      		<xsl:when test="$myparam.upper='KY'"><xsl:value-of select="$myparam"/> (Îles Caïmans)</xsl:when>
      		<xsl:when test="$myparam.upper='KH'"><xsl:value-of select="$myparam"/> (Cambodge)</xsl:when>
      		<xsl:when test="$myparam.upper='CM'"><xsl:value-of select="$myparam"/> (Cameroun)</xsl:when>
      		<xsl:when test="$myparam.upper='CA'"><xsl:value-of select="$myparam"/> (Canada)</xsl:when>
      		<xsl:when test="$myparam.upper='KZ'"><xsl:value-of select="$myparam"/> (Kazakhstan)</xsl:when>
      		<xsl:when test="$myparam.upper='QA'"><xsl:value-of select="$myparam"/> (Qatar)</xsl:when>
      		<xsl:when test="$myparam.upper='KE'"><xsl:value-of select="$myparam"/> (Kenya)</xsl:when>
      		<xsl:when test="$myparam.upper='KG'"><xsl:value-of select="$myparam"/> (Kirghizistan)</xsl:when>
      		<xsl:when test="$myparam.upper='KI'"><xsl:value-of select="$myparam"/> (Kiribati)</xsl:when>
      		<xsl:when test="$myparam.upper='CC'"><xsl:value-of select="$myparam"/> (Îles Cocos (Keeling))</xsl:when>
      		<xsl:when test="$myparam.upper='CO'"><xsl:value-of select="$myparam"/> (Colombie)</xsl:when>
      		<xsl:when test="$myparam.upper='KM'"><xsl:value-of select="$myparam"/> (Comores)</xsl:when>
      		<xsl:when test="$myparam.upper='CG'"><xsl:value-of select="$myparam"/> (Congo)</xsl:when>
      		<xsl:when test="$myparam.upper='HR'"><xsl:value-of select="$myparam"/> (Croatie)</xsl:when>
      		<xsl:when test="$myparam.upper='CU'"><xsl:value-of select="$myparam"/> (Cuba)</xsl:when>
      		<xsl:when test="$myparam.upper='KW'"><xsl:value-of select="$myparam"/> (Koweït)</xsl:when>
      		<xsl:when test="$myparam.upper='LS'"><xsl:value-of select="$myparam"/> (Lesotho)</xsl:when>
      		<xsl:when test="$myparam.upper='LV'"><xsl:value-of select="$myparam"/> (Lettonie)</xsl:when>
      		<xsl:when test="$myparam.upper='LB'"><xsl:value-of select="$myparam"/> (Liban)</xsl:when>
      		<xsl:when test="$myparam.upper='LR'"><xsl:value-of select="$myparam"/> (Liberia)</xsl:when>
      		<xsl:when test="$myparam.upper='LY'"><xsl:value-of select="$myparam"/> (Libye)</xsl:when>
      		<xsl:when test="$myparam.upper='LI'"><xsl:value-of select="$myparam"/> (Liechtenstein)</xsl:when>
      		<xsl:when test="$myparam.upper='LT'"><xsl:value-of select="$myparam"/> (Lituanie)</xsl:when>
      		<xsl:when test="$myparam.upper='LU'"><xsl:value-of select="$myparam"/> (Luxembourg)</xsl:when>
      		<xsl:when test="$myparam.upper='MO'"><xsl:value-of select="$myparam"/> (Macao)</xsl:when>
      		<xsl:when test="$myparam.upper='MG'"><xsl:value-of select="$myparam"/> (Madagascar)</xsl:when>
      		<xsl:when test="$myparam.upper='MW'"><xsl:value-of select="$myparam"/> (Malawi)</xsl:when>
      		<xsl:when test="$myparam.upper='MY'"><xsl:value-of select="$myparam"/> (Malaisie)</xsl:when>
      		<xsl:when test="$myparam.upper='MV'"><xsl:value-of select="$myparam"/> (Maldives)</xsl:when>
      		<xsl:when test="$myparam.upper='ML'"><xsl:value-of select="$myparam"/> (Mali)</xsl:when>
      		<xsl:when test="$myparam.upper='MT'"><xsl:value-of select="$myparam"/> (Malte)</xsl:when>
      		<xsl:when test="$myparam.upper='MP'"><xsl:value-of select="$myparam"/> (Îles Mariannes du Nord)</xsl:when>
      		<xsl:when test="$myparam.upper='MA'"><xsl:value-of select="$myparam"/> (Maroc)</xsl:when>
      		<xsl:when test="$myparam.upper='MH'"><xsl:value-of select="$myparam"/> (Îles Marshall)</xsl:when>
      		<xsl:when test="$myparam.upper='MQ'"><xsl:value-of select="$myparam"/> (Martinique)</xsl:when>
      		<xsl:when test="$myparam.upper='MR'"><xsl:value-of select="$myparam"/> (Mauritanie)</xsl:when>
      		<xsl:when test="$myparam.upper='MU'"><xsl:value-of select="$myparam"/> (Maurice)</xsl:when>
      		<xsl:when test="$myparam.upper='YT'"><xsl:value-of select="$myparam"/> (Mayotte)</xsl:when>
      		<xsl:when test="$myparam.upper='MX'"><xsl:value-of select="$myparam"/> (Mexique)</xsl:when>
      		<xsl:when test="$myparam.upper='MC'"><xsl:value-of select="$myparam"/> (Monaco)</xsl:when>
      		<xsl:when test="$myparam.upper='MN'"><xsl:value-of select="$myparam"/> (Mongolie)</xsl:when>
      		<xsl:when test="$myparam.upper='MS'"><xsl:value-of select="$myparam"/> (Montserrat)</xsl:when>
      		<xsl:when test="$myparam.upper='ME'"><xsl:value-of select="$myparam"/> (Monténégro)</xsl:when>
      		<xsl:when test="$myparam.upper='MZ'"><xsl:value-of select="$myparam"/> (Mozambique)</xsl:when>
      		<xsl:when test="$myparam.upper='MM'"><xsl:value-of select="$myparam"/> (Myanmar)</xsl:when>
      		<xsl:when test="$myparam.upper='NA'"><xsl:value-of select="$myparam"/> (Namibie)</xsl:when>
      		<xsl:when test="$myparam.upper='NR'"><xsl:value-of select="$myparam"/> (Nauru)</xsl:when>
      		<xsl:when test="$myparam.upper='NP'"><xsl:value-of select="$myparam"/> (Népal)</xsl:when>
      		<xsl:when test="$myparam.upper='NC'"><xsl:value-of select="$myparam"/> (Nouvelle-Calédonie)</xsl:when>
      		<xsl:when test="$myparam.upper='NZ'"><xsl:value-of select="$myparam"/> (Nouvelle-Zélande)</xsl:when>
      		<xsl:when test="$myparam.upper='NI'"><xsl:value-of select="$myparam"/> (Nicaragua)</xsl:when>
      		<xsl:when test="$myparam.upper='NL'"><xsl:value-of select="$myparam"/> (Pays-Bas)</xsl:when>
      		<xsl:when test="$myparam.upper='NE'"><xsl:value-of select="$myparam"/> (Niger)</xsl:when>
      		<xsl:when test="$myparam.upper='NG'"><xsl:value-of select="$myparam"/> (Nigeria)</xsl:when>
      		<xsl:when test="$myparam.upper='NU'"><xsl:value-of select="$myparam"/> (Niue)</xsl:when>
      		<xsl:when test="$myparam.upper='NF'"><xsl:value-of select="$myparam"/> (Île Norfolk)</xsl:when>
      		<xsl:when test="$myparam.upper='NO'"><xsl:value-of select="$myparam"/> (Norvège)</xsl:when>
      		<xsl:when test="$myparam.upper='OM'"><xsl:value-of select="$myparam"/> (Oman)</xsl:when>
      		<xsl:when test="$myparam.upper='AT'"><xsl:value-of select="$myparam"/> (Autriche)</xsl:when>
      		<xsl:when test="$myparam.upper='PK'"><xsl:value-of select="$myparam"/> (Pakistan)</xsl:when>
      		<xsl:when test="$myparam.upper='PW'"><xsl:value-of select="$myparam"/> (Palaos)</xsl:when>
      		<xsl:when test="$myparam.upper='PS'"><xsl:value-of select="$myparam"/> (État de Palestine)</xsl:when>
      		<xsl:when test="$myparam.upper='PA'"><xsl:value-of select="$myparam"/> (Panama)</xsl:when>
      		<xsl:when test="$myparam.upper='PG'"><xsl:value-of select="$myparam"/> (Papouasie-Nouvelle-Guinée)</xsl:when>
      		<xsl:when test="$myparam.upper='PY'"><xsl:value-of select="$myparam"/> (Paraguay)</xsl:when>
      		<xsl:when test="$myparam.upper='PE'"><xsl:value-of select="$myparam"/> (Pérou)</xsl:when>
      		<xsl:when test="$myparam.upper='PH'"><xsl:value-of select="$myparam"/> (Philippines)</xsl:when>
      		<xsl:when test="$myparam.upper='PN'"><xsl:value-of select="$myparam"/> (Îles Pitcairn)</xsl:when>
      		<xsl:when test="$myparam.upper='BO'"><xsl:value-of select="$myparam"/> (État plurinational de Bolivie)</xsl:when>
      		<xsl:when test="$myparam.upper='PL'"><xsl:value-of select="$myparam"/> (Pologne)</xsl:when>
      		<xsl:when test="$myparam.upper='PT'"><xsl:value-of select="$myparam"/> (Portugal)</xsl:when>
      		<xsl:when test="$myparam.upper='PR'"><xsl:value-of select="$myparam"/> (Porto Rico)</xsl:when>
      		<xsl:when test="$myparam.upper='KR'"><xsl:value-of select="$myparam"/> (République de Corée)</xsl:when>
      		<xsl:when test="$myparam.upper='MD'"><xsl:value-of select="$myparam"/> (République de Moldavie)</xsl:when>
      		<xsl:when test="$myparam.upper='RE'"><xsl:value-of select="$myparam"/> (La Réunion)</xsl:when>
      		<xsl:when test="$myparam.upper='RW'"><xsl:value-of select="$myparam"/> (Rwanda)</xsl:when>
      		<xsl:when test="$myparam.upper='RO'"><xsl:value-of select="$myparam"/> (Roumanie)</xsl:when>
      		<xsl:when test="$myparam.upper='RU'"><xsl:value-of select="$myparam"/> (Fédération de Russie)</xsl:when>
      		<xsl:when test="$myparam.upper='SB'"><xsl:value-of select="$myparam"/> (Îles Salomon)</xsl:when>
      		<xsl:when test="$myparam.upper='ZM'"><xsl:value-of select="$myparam"/> (Zambie)</xsl:when>
      		<xsl:when test="$myparam.upper='WS'"><xsl:value-of select="$myparam"/> (Samoa)</xsl:when>
      		<xsl:when test="$myparam.upper='SM'"><xsl:value-of select="$myparam"/> (Saint-Marin)</xsl:when>
      		<xsl:when test="$myparam.upper='ST'"><xsl:value-of select="$myparam"/> (Sao Tomé-et-Príncipe)</xsl:when>
      		<xsl:when test="$myparam.upper='SA'"><xsl:value-of select="$myparam"/> (Arabie saoudite)</xsl:when>
      		<xsl:when test="$myparam.upper='SE'"><xsl:value-of select="$myparam"/> (Suède)</xsl:when>
      		<xsl:when test="$myparam.upper='CH'"><xsl:value-of select="$myparam"/> (Suisse)</xsl:when>
      		<xsl:when test="$myparam.upper='SN'"><xsl:value-of select="$myparam"/> (Sénégal)</xsl:when>
      		<xsl:when test="$myparam.upper='RS'"><xsl:value-of select="$myparam"/> (Serbie)</xsl:when>
      		<xsl:when test="$myparam.upper='SC'"><xsl:value-of select="$myparam"/> (Seychelles)</xsl:when>
      		<xsl:when test="$myparam.upper='SL'"><xsl:value-of select="$myparam"/> (Sierra Leone)</xsl:when>
      		<xsl:when test="$myparam.upper='ZW'"><xsl:value-of select="$myparam"/> (Zimbabwe)</xsl:when>
      		<xsl:when test="$myparam.upper='SG'"><xsl:value-of select="$myparam"/> (Singapour)</xsl:when>
      		<xsl:when test="$myparam.upper='SK'"><xsl:value-of select="$myparam"/> (Slovaquie)</xsl:when>
      		<xsl:when test="$myparam.upper='SI'"><xsl:value-of select="$myparam"/> (Slovénie)</xsl:when>
      		<xsl:when test="$myparam.upper='SO'"><xsl:value-of select="$myparam"/> (Somalie)</xsl:when>
      		<xsl:when test="$myparam.upper='ES'"><xsl:value-of select="$myparam"/> (Espagne)</xsl:when>
      		<xsl:when test="$myparam.upper='LK'"><xsl:value-of select="$myparam"/> (Sri Lanka)</xsl:when>
      		<xsl:when test="$myparam.upper='BL'"><xsl:value-of select="$myparam"/> (Saint-Barthélemy)</xsl:when>
      		<xsl:when test="$myparam.upper='SH'"><xsl:value-of select="$myparam"/> (Sainte-Hélène, Ascension et Tristan da Cunha)</xsl:when>
      		<xsl:when test="$myparam.upper='KN'"><xsl:value-of select="$myparam"/> (Saint-Kitts-et-Nevis)</xsl:when>
      		<xsl:when test="$myparam.upper='LC'"><xsl:value-of select="$myparam"/> (Sainte-Lucie)</xsl:when>
      		<xsl:when test="$myparam.upper='MF'"><xsl:value-of select="$myparam"/> (Saint-Martin (partie française))</xsl:when>
      		<xsl:when test="$myparam.upper='SX'"><xsl:value-of select="$myparam"/> (Saint-Martin (partie néerlandaise))</xsl:when>
      		<xsl:when test="$myparam.upper='PM'"><xsl:value-of select="$myparam"/> (Saint-Pierre-et-Miquelon)</xsl:when>
      		<xsl:when test="$myparam.upper='VC'"><xsl:value-of select="$myparam"/> (Saint-Vincent-et-les-Grenadines)</xsl:when>
      		<xsl:when test="$myparam.upper='ZA'"><xsl:value-of select="$myparam"/> (Afrique du Sud)</xsl:when>
      		<xsl:when test="$myparam.upper='SD'"><xsl:value-of select="$myparam"/> (Soudan)</xsl:when>
      		<xsl:when test="$myparam.upper='GS'"><xsl:value-of select="$myparam"/> (Géorgie du Sud-et-les Îles Sandwich du Sud)</xsl:when>
      		<xsl:when test="$myparam.upper='SS'"><xsl:value-of select="$myparam"/> (Soudan du Sud)</xsl:when>
      		<xsl:when test="$myparam.upper='SR'"><xsl:value-of select="$myparam"/> (Suriname)</xsl:when>
      		<xsl:when test="$myparam.upper='SJ'"><xsl:value-of select="$myparam"/> (Svalbard et Jan Mayen)</xsl:when>
      		<xsl:when test="$myparam.upper='SZ'"><xsl:value-of select="$myparam"/> (Eswatini)</xsl:when>
      		<xsl:when test="$myparam.upper='TJ'"><xsl:value-of select="$myparam"/> (Tadjikistan)</xsl:when>
      		<xsl:when test="$myparam.upper='TW'"><xsl:value-of select="$myparam"/> (Taïwan)</xsl:when>
      		<xsl:when test="$myparam.upper='TH'"><xsl:value-of select="$myparam"/> (Thaïlande)</xsl:when>
      		<xsl:when test="$myparam.upper='TL'"><xsl:value-of select="$myparam"/> (Timor-Leste)</xsl:when>
      		<xsl:when test="$myparam.upper='TG'"><xsl:value-of select="$myparam"/> (Togo)</xsl:when>
      		<xsl:when test="$myparam.upper='TK'"><xsl:value-of select="$myparam"/> (Tokelau)</xsl:when>
      		<xsl:when test="$myparam.upper='TO'"><xsl:value-of select="$myparam"/> (Tonga)</xsl:when>
      		<xsl:when test="$myparam.upper='TT'"><xsl:value-of select="$myparam"/> (Trinité-et-Tobago)</xsl:when>
      		<xsl:when test="$myparam.upper='TD'"><xsl:value-of select="$myparam"/> (Tchad)</xsl:when>
      		<xsl:when test="$myparam.upper='CZ'"><xsl:value-of select="$myparam"/> (Tchéquie)</xsl:when>
      		<xsl:when test="$myparam.upper='TN'"><xsl:value-of select="$myparam"/> (Tunisie)</xsl:when>
      		<xsl:when test="$myparam.upper='TR'"><xsl:value-of select="$myparam"/> (Turquie)</xsl:when>
      		<xsl:when test="$myparam.upper='TM'"><xsl:value-of select="$myparam"/> (Turkménistan)</xsl:when>
      		<xsl:when test="$myparam.upper='TC'"><xsl:value-of select="$myparam"/> (Îles Turques-et-Caïques)</xsl:when>
      		<xsl:when test="$myparam.upper='TV'"><xsl:value-of select="$myparam"/> (Tuvalu)</xsl:when>
      		<xsl:when test="$myparam.upper='UG'"><xsl:value-of select="$myparam"/> (Ouganda)</xsl:when>
      		<xsl:when test="$myparam.upper='UA'"><xsl:value-of select="$myparam"/> (Ukraine)</xsl:when>
      		<xsl:when test="$myparam.upper='HU'"><xsl:value-of select="$myparam"/> (Hongrie)</xsl:when>
      		<xsl:when test="$myparam.upper='UY'"><xsl:value-of select="$myparam"/> (Uruguay)</xsl:when>
      		<xsl:when test="$myparam.upper='UZ'"><xsl:value-of select="$myparam"/> (Ouzbékistan)</xsl:when>
      		<xsl:when test="$myparam.upper='VU'"><xsl:value-of select="$myparam"/> (Vanuatu)</xsl:when>
      		<xsl:when test="$myparam.upper='VA'"><xsl:value-of select="$myparam"/> (Saint-Siège)</xsl:when>
      		<xsl:when test="$myparam.upper='AE'"><xsl:value-of select="$myparam"/> (Émirats arabes unis)</xsl:when>
      		<xsl:when test="$myparam.upper='TZ'"><xsl:value-of select="$myparam"/> (République-Unie de Tanzanie)</xsl:when>
      		<xsl:when test="$myparam.upper='US'"><xsl:value-of select="$myparam"/> (États-Unis)</xsl:when>
      		<xsl:when test="$myparam.upper='GB'"><xsl:value-of select="$myparam"/> (Royaume-Uni)</xsl:when>
      		<xsl:when test="$myparam.upper='VN'"><xsl:value-of select="$myparam"/> (Viêt Nam)</xsl:when>
      		<xsl:when test="$myparam.upper='WF'"><xsl:value-of select="$myparam"/> (Wallis-et-Futuna)</xsl:when>
      		<xsl:when test="$myparam.upper='CX'"><xsl:value-of select="$myparam"/> (Île Christmas)</xsl:when>
      		<xsl:when test="$myparam.upper='BY'"><xsl:value-of select="$myparam"/> (Biélorussie)</xsl:when>
      		<xsl:when test="$myparam.upper='EH'"><xsl:value-of select="$myparam"/> (Sahara occidental)</xsl:when>
      		<xsl:when test="$myparam.upper='CF'"><xsl:value-of select="$myparam"/> (République centrafricaine)</xsl:when>
      		<xsl:when test="$myparam.upper='CY'"><xsl:value-of select="$myparam"/> (Chypre)</xsl:when>
   			<xsl:otherwise><xsl:value-of select="$myparam"/></xsl:otherwise>
		</xsl:choose>
	</xsl:template>

</xsl:stylesheet>