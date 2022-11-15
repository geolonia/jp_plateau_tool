# `建築物::住所`

```
			<bldg:address>
				<core:Address>
					<core:xalAddress>
						<xAL:AddressDetails>
							<xAL:Country>
								<xAL:CountryName>日本</xAL:CountryName>
								<xAL:Locality>
									<xAL:LocalityName Type="Town">東京都文京区大塚三丁目</xAL:LocalityName>
								</xAL:Locality>
							</xAL:Country>
						</xAL:AddressDetails>
					</core:xalAddress>
				</core:Address>
			</bldg:address>
```

# `建築物::建物利用現況`

* `図上面積` -> `uro:buildingRoofEdgeArea`
* `地域地区` -> `uro:districtsAndZonesType`
* `都道府県` -> `uro:prefecture`
* `市区町村` -> `uro:city`
* `調査年` -> `uro:surveyYear`

```
			<uro:buildingDetails>
				<uro:BuildingDetails>
					<uro:buildingRoofEdgeArea uom="m2">3433.94615</uro:buildingRoofEdgeArea>
					<uro:districtsAndZonesType codeSpace="../../codelists/Common_districtsAndZonesType.xml">3</uro:districtsAndZonesType>
					<uro:prefecture codeSpace="../../codelists/Common_prefecture.xml">13</uro:prefecture>
					<uro:city codeSpace="../../codelists/Common_localPublicAuthorities.xml">13105</uro:city>
					<uro:surveyYear>2016</uro:surveyYear>
				</uro:BuildingDetails>
			</uro:buildingDetails>
```

# `建築物::拡張属性::調査年::key=2`

```
			<uro:extendedAttribute>
				<uro:KeyValuePair>
					<uro:key codeSpace="../../codelists/extendedAttribute_key.xml">2</uro:key>
					<uro:codeValue codeSpace="../../codelists/extendedAttribute_key2.xml">2</uro:codeValue>
				</uro:KeyValuePair>
			</uro:extendedAttribute>
```

key=2 is `LOD1の立ち上げに使用する建築物の高さ`

内容

```
	<gml:Dictionary xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:gml="http://www.opengis.net/gml" xsi:schemaLocation="http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/profiles/SimpleDictionary/1.0.0/gmlSimpleDictionaryProfile.xsd" gml:id="ExtendedAttribute_key2">
		<gml:name>ExtendedAttribute_key2</gml:name>
		<gml:dictionaryEntry>
			<gml:Definition gml:id="id1">
				<gml:description>点群から取得_最高高さ</gml:description>
				<gml:name>1</gml:name>
			</gml:Definition>
		</gml:dictionaryEntry>
		<gml:dictionaryEntry>
			<gml:Definition gml:id="id2">
				<gml:description>点群から取得_中央値</gml:description>
				<gml:name>2</gml:name>
			</gml:Definition>
		</gml:dictionaryEntry>
		<gml:dictionaryEntry>
			<gml:Definition gml:id="id3">
				<gml:description>点群から取得_平均値</gml:description>
				<gml:name>3</gml:name>
			</gml:Definition>
		</gml:dictionaryEntry>
			<gml:dictionaryEntry>
			<gml:Definition gml:id="id4">
				<gml:description>点群から取得_最頻値</gml:description>
				<gml:name>4</gml:name>
			</gml:Definition>
		</gml:dictionaryEntry>
			<gml:dictionaryEntry>
			<gml:Definition gml:id="id5">
				<gml:description>点群から取得_最低値</gml:description>
				<gml:name>5</gml:name>
			</gml:Definition>
		</gml:dictionaryEntry>
			<gml:dictionaryEntry>
			<gml:Definition gml:id="id6">
				<gml:description>航空写真図化_最高高さ</gml:description>
				<gml:name>6</gml:name>
			</gml:Definition>
		</gml:dictionaryEntry>
			<gml:dictionaryEntry>
			<gml:Definition gml:id="id7">
				<gml:description>建築確認申請書類等に記載された「建築物の高さ」</gml:description>
				<gml:name>7</gml:name>
			</gml:Definition>
		</gml:dictionaryEntry>
	</gml:Dictionary>
```

# `建築物::汎用属性::建物ID`

```
			<gen:stringAttribute name="建物ID">
				<gen:value>13105-bldg-19633</gen:value>
			</gen:stringAttribute>
```

# `建築物::災害リスク汎用属性セット::*`

```
			<gen:genericAttributeSet name="神田川流域浸水予想区域（想定最大規模）">
				<gen:stringAttribute name="規模">
					<gen:value>L2</gen:value>
				</gen:stringAttribute>
				<gen:stringAttribute name="浸水ランク">
					<gen:value>1</gen:value>
				</gen:stringAttribute>
				<gen:measureAttribute name="浸水深">
					<gen:value uom="m">0.042</gen:value>
				</gen:measureAttribute>
			</gen:genericAttributeSet>
```

