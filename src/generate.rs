/* SPDX-License-Identifier: CC0-1.0
 *
 * src/generate.rs
 *
 * This file is a component of ShadyURL by Elizabeth Myers.
 *
 * To the extent possible under law, the person who associated CC0 with
 * ShadyURL has waived all copyright and related or neighboring rights
 * to ShadyURL.
 *
 * You should have received a copy of the CC0 legalcode along with this
 * work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
 */

use rand::{
    distributions::{Distribution, Uniform},
    prelude::*,
};

const NSFW: &[&str] = &[
"0-percent-risk",
    "100-percent-legal",
    "1-dollar-iphone",
    "1man1jar",
    "2girls1cup",
    "2guys1horse",
    "3girlsfingerpaint",
    "3guys1hammer",
    "419-scam",
    "420",
    "500-dollar-cash-prize",
    "69",
    "7-11-robbery",
    "800-dollars-4u",
    "9-11-jumper",
    "9-11-video",
    "abuse",
    "adblock-bypass",
    "admin",
    "ads",
    "advert",
    "affair",
    "alien-abduction",
    "ambien",
    "america",
    "anal",
    "anal-penetration",
    "anal-sex",
    "anarchist",
    "anarchy",
    "antifa",
    "antivirus",
    "apple-giveaway",
    "arpa",
    "ar-15",
    "ass",
    "ass-beating",
    "audio",
    "babes",
    "bad-mixtape",
    "badonkadonk",
    "bang",
    "bank-transfer",
    "barely-legal",
    "bbw",
    "bdsm",
    "beatdown",
    "beating-women",
    "beheading",
    "big-bang",
    "big-cash-prize",
    "big-cocks",
    "big-milf",
    "big-willie",
    "billionaire",
    "bitcoin-2x",
    "bitcoin-billionaire",
    "bitcoin-cash-paydirt",
    "bitcoin-miner",
    "bitcoin-multiplier",
    "blood",
    "bloody-murder",
    "bodies",
    "body",
    "bodybuilders",
    "bomb",
    "bombing",
    "bondage",
    "bot",
    "boyfriend",
    "boyfriend-camera",
    "boyfriend-phone",
    "boyfriend-tracker",
    "bribe-your-boss-legally",
    "build-muscle",
    "bupropion",
    "butt",
    "buttsex",
    "bypass",
    "calcium",
    "caliberal",
    "california",
    "cash",
    "cash-money",
    "casino",
    "casino-how-2-win",
    "casino-loosest-slots",
    "celeb-addresses",
    "celeb-phone-numbers",
    "celeb-sextape",
    "chat",
    "chat-with-babes",
    "cheap",
    "cheap-cialis",
    "cheap-drugs",
    "cheap-pills",
    "cheap-viagra",
    "cheat",
    "cheat-at-casino-legally",
    "cheating",
    "chop",
    "christian",
    "chromium-supplement",
    "cialis",
    "classified",
    "classified-doxxx",
    "click",
    "click-here",
    "clone-phone",
    "clone-sim",
    "cobalt-supplement",
    "cocaine",
    "coin",
    "coin-multiplier",
    "communism",
    "communist",
    "conservative",
    "conspiracy",
    "coupon",
    "cowgirl",
    "crack",
    "credit-card",
    "credit-card-numbers",
    "crime-tips",
    "crypto",
    "cummy",
    "daesh",
    "daesh-meetup",
    "daesh-recruitment",
    "darkweb",
    "dead",
    "deagle",
    "death",
    "deepfake",
    "deepweb",
    "desert-eagle",
    "detector",
    "diaper-kink",
    "diazepam",
    "diet",
    "diet-supplement",
    "diet-fast-weight-loss",
    "dns",
    "doctor",
    "doctor-recommended",
    "dog-sex",
    "dollar",
    "dollar-forex",
    "domestic-abuse",
    "donate",
    "donkey-cock",
    "doxxing",
    "doxxx",
    "do-this-by-tomorrow",
    "do-this-now",
    "drive-by-shooting",
    "drugs",
    "dungeon",
    "earn-ur-degree-online",
    "effexor",
    "election-fraud",
    "email",
    "escort",
    "estradiol",
    "etherium",
    "etherium-multiplier",
    "euro",
    "euro-forex",
    "evangelical",
    "evidence",
    "exclusive-sex-tape",
    "execute",
    "extra",
    "fake",
    "fap",
    "fappening",
    "famine",
    "fast-weight-loss",
    "final-moments",
    "final-opportunity",
    "finder",
    "fingering",
    "fisting",
    "flash",
    "flight-points",
    "forex",
    "forex-interbank",
    "forex-no-bear-market",
    "fraud",
    "freakout",
    "free",
    "free-android",
    "free-internet",
    "free-iphone",
    "free-meds",
    "free-pills",
    "free-phone",
    "free-porn",
    "free-shows",
    "free-tv",
    "free-webcams",
    "french",
    "frottage",
    "fuck",
    "gangbang",
    "gay",
    "gay-sex",
    "german",
    "get-jacked",
    "get-laid-tonite",
    "get-rich-overnight",
    "get-rich-quick",
    "ghetto",
    "ginger",
    "girl",
    "girlcock",
    "girldick",
    "girlfriend",
    "girlfriend-camera",
    "girlfriend-phone",
    "girlfriend-tracker",
    "giveaway",
    "gmail",
    "goatse",
    "gone-wild",
    "gone-wrong",
    "google",
    "gore",
    "government-documents",
    "government-dox",
    "gps",
    "gps-location",
    "gps-track",
    "grindr",
    "guns",
    "gun-fight",
    "hacker",
    "helicopter-crash",
    "he-dies",
    "he-fux-her",
    "headshot",
    "heroin",
    "hidden",
    "hijack",
    "hijacker",
    "holistic-medicine",
    "homeless-death",
    "homeless-man",
    "homeless-woman",
    "homeopathic-remedy",
    "horny-goat-weed",
    "horny-teens",
    "horny-women",
    "hood-shooting",
    "horse-sex",
    "hotmail",
    "hot-babes",
    "hot-women",
    "how-to-steal",
    "how-to-win",
    "huge-cash-prize",
    "huge-cocks",
    "hypnosis",
    "ied",
    "i-love-u",
    "i-make-2000-week-at-home",
    "i-was-abducted-by-aliens",
    "i-was-probed",
    "illegal",
    "illegal-porno",
    "impersonate",
    "increase-ur-t",
    "india",
    "install",
    "internet",
    "interview",
    "intifada",
    "invis",
    "ip-finder",
    "ip-hijacker",
    "ip-stealer",
    "iron",
    "irs",
    "isis",
    "isis-recruiter",
    "isis-training-camp",
    "islam",
    "israel",
    "jackpot",
    "jackpot-lottry-winner",
    "jailbait",
    "jailbreak",
    "java",
    "jihad",
    "john",
    "journalist",
    "k9",
    "keygen",
    "keylog",
    "keylogger",
    "king",
    "king-scandal",
    "kinky",
    "kitty",
    "knife-fight",
    "leak",
    "leaked-documents",
    "leaked-dox",
    "legal",
    "lemonparty",
    "lesbian",
    "lesbians-fuck",
    "lesbo",
    "levitra",
    "lexapro",
    "lezbo",
    "liberal",
    "linux",
    "local",
    "local-men",
    "local-women",
    "locator",
    "loli",
    "lolicon",
    "lonely-women",
    "lorazepam",
    "lsd",
    "lotto-winner",
    "macos",
    "maga",
    "make-her-cum",
    "make-money-at-home",
    "make-money-doing-nothing",
    "male-enhancement",
    "malware",
    "marijuana",
    "mature",
    "mdma",
    "mega",
    "mega-jackpot",
    "mega-tits",
    "microsoft-giveaway",
    "mike-pence",
    "mike-pence-gay",
    "mike-pence-naked",
    "milf",
    "milf-tits",
    "millionaire",
    "mine-coins-for-free",
    "mommy-milkers",
    "monster-erection",
    "multivitamin",
    "muslim",
    "my-tits-are-legend",
    "naked",
    "naked-celebs",
    "nazi-beatdown",
    "new-world-order",
    "nft",
    "nigeria-bank-transfer",
    "no-risk",
    "nudes",
    "nwo",
    "old-ladies",
    "old-man-gangbang",
    "old-men",
    "old-remedy",
    "only-10-dollars",
    "orangutan-sex",
    "organ",
    "organ-selling",
    "overnight",
    "overnight-wealth",
    "overnite-success",
    "overnite-billionaire",
    "overnite-millionaire",
    "overnite-trillionaire",
    "password",
    "penetration",
    "penis-enlargement",
    "petit-milf",
    "pewdiepie-sex-tape",
    "phishing",
    "phone",
    "phone-numbers",
    "phosphorous-supplements",
    "physical-removal",
    "pickup",
    "pickup-girls",
    "pig-sex",
    "pills",
    "pills-4-cheap",
    "pimp",
    "pirated-movies",
    "pirated-music",
    "pirated-podcasts",
    "pirated-shows",
    "pizzagate",
    "poker",
    "police",
    "police-bodycam",
    "poop",
    "popup",
    "prince",
    "prince-scandal",
    "princess",
    "princess-scandal",
    "probe",
    "probing",
    "protein",
    "protein-powder",
    "prozac",
    "pub(crate)lic-sex",
    "pussy",
    "pussy-fuck",
    "putin",
    "putins-dick",
    "putin-naked",
    "pwn",
    "qanon",
    "qanon-reveal",
    "queen",
    "queen-nudes",
    "queen-scandal",
    "quit-ur-job",
    "rapid-growth",
    "rapid-weight-loss",
    "rat-sex",
    "read",
    "real",
    "really-horny-girls",
    "remote",
    "remote-viewing",
    "reverse",
    "rich",
    "rich-overnight",
    "ripoff",
    "risperidol",
    "root",
    "rootkit",
    "russian",
    "russian-bots",
    "scam",
    "scat",
    "school-shooting",
    "scissoring",
    "seattle",
    "secret",
    "secret-plans",
    "secretary",
    "see-her-webcam",
    "see-my-pussy",
    "see-my-tits",
    "sell-your-organs",
    "seroquel",
    "sex",
    "sexy",
    "sexy-ladies",
    "sexy-women",
    "sex-offender",
    "sex-tape",
    "sexwithcats",
    "sexwithdogs",
    "sexwithgerbils",
    "sim-clone",
    "she-dies",
    "she-fux-him",
    "she-will-never-know",
    "shes-waiting-4u",
    "shit",
    "shooting",
    "shoplifting",
    "shota",
    "shotacon",
    "slut",
    "sms",
    "snuff",
    "social-security",
    "socialism",
    "socialist",
    "sound",
    "source-code",
    "source-code-leak",
    "spam",
    "spy",
    "spy-on-ur-boyfriend",
    "spy-on-ur-girlfriend",
    "spy-on-ur-husband",
    "spy-on-ur-wife",
    "spyware",
    "stalk-her",
    "steal",
    "stoned",
    "stoned-for-adultery",
    "stud",
    "stuxnet",
    "st-johns-wort",
    "subway-death",
    "supplements",
    "supplements-cheap",
    "suppository",
    "swiss-bank-account",
    "swiss-lotto-winner",
    "takeover",
    "taliban-interview",
    "taliban-meetup",
    "taliban-recruiter",
    "teen",
    "teen-barely-legal",
    "telegram",
    "televangelist",
    "terrorist",
    "testosterone",
    "testosterone-supplement",
    "the-donald",
    "they-cant-stop-you",
    "they-hate-this",
    "they-hurt-her",
    "tighten-my-pussy",
    "tokens",
    "tor",
    "torrent",
    "track-my-ex",
    "track-my-wife",
    "tracker",
    "trans",
    "trans-agenda",
    "trans-doctor",
    "trillionaire",
    "trojan",
    "underground-death",
    "unlocker",
    "un-conspiracy",
    "ur-hubby-is-cumming",
    "ur-jackpot-awaits",
    "ur-wife-is-cumming",
    "usa",
    "vagina",
    "vanadium-supplement",
    "viagra",
    "video",
    "virus",
    "vitamins",
    "vitamin-b12",
    "vitamin-b6",
    "vitamin-c",
    "vitamin-d",
    "vitamin-e",
    "vpn",
    "vuln",
    "vulns",
    "war-photos",
    "warez",
    "watch",
    "watch-her-die",
    "watch-her-get-fukd",
    "watch-her-webcam",
    "watch-him-die",
    "watch-movies-4-free",
    "watch-tv-for-free",
    "web",
    "weed",
    "weight-loss",
    "wet",
    "wet-pussy",
    "wfh",
    "what-hes-doing",
    "what-shes-doing",
    "what-they-dont-want-u2-know",
    "whos-my-husband-txting",
    "whos-my-wife-been-txting",
    "wife-pussy",
    "win",
    "win-at-blackjack",
    "win-at-poker",
    "win-big",
    "windows",
    "winning",
    "winning-lotto-numbers",
    "woman-beaten",
    "women",
    "woman-gets-stoned",
    "work-at-home",
    "worldstar",
    "worldstar-fight",
    "x",
    "xtc",
    "xxx",
    "xxx-pussy",
    "xxx-tits",
    "xxxtra",
    "yahoo",
    "yen",
    "yen-forex",
    "your-ip",
    "your-ssn",
    "youtube-download",
    "zinc-supplement",
    "zoo",
    "zoo-porn",
    "zoom",
];

const EXT: &[&str] = &[
    ".app", ".avi", ".bat", ".csv", ".divx", ".dll", ".doc", ".docx", ".exe", ".flv", ".gif",
    ".htm", ".html", ".hxt", ".ini", ".jar", ".jpg", ".m1v", ".m4a", ".mid", ".midi", ".mkv",
    ".mov", ".movie", ".mpa", ".mpe", ".mpeg", ".mpg", ".mp3", ".mp4", ".msi", ".p7r", ".pdf",
    ".png", ".ppt", ".pptx", ".rar", ".snd", ".txt", ".vbs", ".xaf", ".xls", ".xlsx", ".xml",
    ".zip",
];

fn generate_hash(rng: &mut dyn RngCore) -> String {
    const CHARS: &[u8] = b"abcdefghijiklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_+";

    let between = Uniform::from(0..CHARS.len());
    let char_count = rng.gen_range(8..16);
    let mut ret = String::with_capacity(char_count);
    for _ in 0..char_count {
        ret.push(CHARS[between.sample(rng)].into());
    }
    ret
}

pub(crate) fn shady_filename(rng: &mut dyn RngCore) -> String {
    let between_nsfw = Uniform::from(0..NSFW.len());
    let nsfw_count = rng.gen_range(4..10);
    let mut out = Vec::new();
    let mut has_hash = false;
    for _ in 0..nsfw_count {
        if !has_hash && rng.gen_range(0..nsfw_count) == 0 {
            out.push(generate_hash(rng));
            has_hash = true;
        } else {
            out.push(NSFW[between_nsfw.sample(rng)].to_string());
        }
    }

    if !has_hash {
        out.push(generate_hash(rng));
    }

    let mut string = out.join("-");
    string.push_str(EXT[rng.gen_range(0..EXT.len())]);
    string
}
