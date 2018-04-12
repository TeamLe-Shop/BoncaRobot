extern crate csv;
#[macro_use]
extern crate plugin_api;

use plugin_api::prelude::*;
use std::collections::HashMap;

struct IsoLangPlugin {
    map: HashMap<String, String>,
}

impl IsoLangPlugin {
    fn isolang(this: &mut Plugin, arg: &str, ctx: Context) {
        let this: &mut Self = this.downcast_mut().unwrap();
        let response = this.map
            .get(arg)
            .cloned()
            .or_else(|| {
                for (k, v) in &this.map {
                    if v.to_lowercase().contains(&arg.to_lowercase()) {
                        return Some(format!("{} => {}", v, k));
                    }
                }
                None
            })
            .unwrap_or_else(|| "ISO MOTHERFUCKER, DO YOU SPEAK IT?".into());
        ctx.send_channel(&format!("{}: {}", ctx.sender.nickname(), response));
    }
}

impl Plugin for IsoLangPlugin {
    fn new() -> Self {
        let mut rdr = csv::Reader::from_string(CSV);
        let mut map = HashMap::new();

        for row in rdr.decode() {
            let (code, name): (String, String) = row.unwrap();
            map.insert(code, name);
        }
        Self { map }
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command(
            "isolang",
            "Shows the full names of 2 character ISO lang codes",
            Self::isolang,
        );
    }
}

plugin_export!(IsoLangPlugin);

const CSV: &str = r#"
"alpha2","English"
"aa","Afar"
"ab","Abkhazian"
"ae","Avestan"
"af","Afrikaans"
"ak","Akan"
"am","Amharic"
"an","Aragonese"
"ar","Arabic"
"as","Assamese"
"av","Avaric"
"ay","Aymara"
"az","Azerbaijani"
"ba","Bashkir"
"be","Belarusian"
"bg","Bulgarian"
"bh","Bihari languages"
"bi","Bislama"
"bm","Bambara"
"bn","Bengali"
"bo","Tibetan"
"br","Breton"
"bs","Bosnian"
"ca","Catalan; Valencian"
"ce","Chechen"
"ch","Chamorro"
"co","Corsican"
"cr","Cree"
"cs","Czech"
"cu","Church Slavic; Old Slavonic; Church Slavonic; Old Bulgarian; Old Church Slavonic"
"cv","Chuvash"
"cy","Welsh"
"da","Danish"
"de","German"
"dv","Divehi; Dhivehi; Maldivian"
"dz","Dzongkha"
"ee","Ewe"
"el","Greek, Modern (1453-)"
"en","English"
"eo","Esperanto"
"es","Spanish; Castilian"
"et","Estonian"
"eu","Basque"
"fa","Persian"
"ff","Fulah"
"fi","Finnish"
"fj","Fijian"
"fo","Faroese"
"fr","French"
"fy","Western Frisian"
"ga","Irish"
"gd","Gaelic; Scottish Gaelic"
"gl","Galician"
"gn","Guarani"
"gu","Gujarati"
"gv","Manx"
"ha","Hausa"
"he","Hebrew"
"hi","Hindi"
"ho","Hiri Motu"
"hr","Croatian"
"ht","Haitian; Haitian Creole"
"hu","Hungarian"
"hy","Armenian"
"hz","Herero"
"ia","Interlingua (International Auxiliary Language Association)"
"id","Indonesian"
"ie","Interlingue; Occidental"
"ig","Igbo"
"ii","Sichuan Yi; Nuosu"
"ik","Inupiaq"
"io","Ido"
"is","Icelandic"
"it","Italian"
"iu","Inuktitut"
"ja","Japanese"
"jv","Javanese"
"ka","Georgian"
"kg","Kongo"
"ki","Kikuyu; Gikuyu"
"kj","Kuanyama; Kwanyama"
"kk","Kazakh"
"kl","Kalaallisut; Greenlandic"
"km","Central Khmer"
"kn","Kannada"
"ko","Korean"
"kr","Kanuri"
"ks","Kashmiri"
"ku","Kurdish"
"kv","Komi"
"kw","Cornish"
"ky","Kirghiz; Kyrgyz"
"la","Latin"
"lb","Luxembourgish; Letzeburgesch"
"lg","Ganda"
"li","Limburgan; Limburger; Limburgish"
"ln","Lingala"
"lo","Lao"
"lt","Lithuanian"
"lu","Luba-Katanga"
"lv","Latvian"
"mg","Malagasy"
"mh","Marshallese"
"mi","Maori"
"mk","Macedonian"
"ml","Malayalam"
"mn","Mongolian"
"mr","Marathi"
"ms","Malay"
"mt","Maltese"
"my","Burmese"
"na","Nauru"
"nb","Bokmål, Norwegian; Norwegian Bokmål"
"nd","Ndebele, North; North Ndebele"
"ne","Nepali"
"ng","Ndonga"
"nl","Dutch; Flemish"
"nn","Norwegian Nynorsk; Nynorsk, Norwegian"
"no","Norwegian"
"nr","Ndebele, South; South Ndebele"
"nv","Navajo; Navaho"
"ny","Chichewa; Chewa; Nyanja"
"oc","Occitan (post 1500); Provençal"
"oj","Ojibwa"
"om","Oromo"
"or","Oriya"
"os","Ossetian; Ossetic"
"pa","Panjabi; Punjabi"
"pi","Pali"
"pl","Polish"
"ps","Pushto; Pashto"
"pt","Portuguese"
"qu","Quechua"
"rm","Romansh"
"rn","Rundi"
"ro","Romanian; Moldavian; Moldovan"
"ru","Russian"
"rw","Kinyarwanda"
"sa","Sanskrit"
"sc","Sardinian"
"sd","Sindhi"
"se","Northern Sami"
"sg","Sango"
"si","Sinhala; Sinhalese"
"sk","Slovak"
"sl","Slovenian"
"sm","Samoan"
"sn","Shona"
"so","Somali"
"sq","Albanian"
"sr","Serbian"
"ss","Swati"
"st","Sotho, Southern"
"su","Sundanese"
"sv","Swedish"
"sw","Swahili"
"ta","Tamil"
"te","Telugu"
"tg","Tajik"
"th","Thai"
"ti","Tigrinya"
"tk","Turkmen"
"tl","Tagalog"
"tn","Tswana"
"to","Tonga (Tonga Islands)"
"tr","Turkish"
"ts","Tsonga"
"tt","Tatar"
"tw","Twi"
"ty","Tahitian"
"ug","Uighur; Uyghur"
"uk","Ukrainian"
"ur","Urdu"
"uz","Uzbek"
"ve","Venda"
"vi","Vietnamese"
"vo","Volapük"
"wa","Walloon"
"wo","Wolof"
"xh","Xhosa"
"yi","Yiddish"
"yo","Yoruba"
"za","Zhuang; Chuang"
"zh","Chinese"
"zu","Zulu"
"#;
