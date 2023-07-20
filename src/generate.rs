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

use std::collections::HashSet;

use once_cell::sync::Lazy;
use rand::{
    distributions::{Distribution, Uniform},
    prelude::*,
};

use crate::util::macros::arr;

#[derive(PartialEq, Eq, Copy, Clone)]
enum Mangler {
    NoOp,
    RandomUppercase,
    AllUppercase,
    ReplaceSeps,
}

fn generate_hash(rng: &mut dyn RngCore) -> String {
    arr!(const CHARS: [u8; _] = *b"abcdefghijiklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_+$");
    let distr_chars = Lazy::new(|| Uniform::new(0, CHARS.len()));
    let distr_count = Lazy::new(|| Uniform::new_inclusive(8, 16));

    let char_count = distr_count.sample(rng);
    (0..char_count)
        .map(|_| CHARS[distr_chars.sample(rng)] as char)
        .collect()
}

fn perform_mangle(fragment: &str, mangler: Mangler, rng: &mut dyn RngCore) -> String {
    match mangler {
        Mangler::RandomUppercase => fragment
            .chars()
            .map(|ch| {
                let distr_cap = Lazy::new(|| Uniform::new(0, 3));
                if (*distr_cap).sample(rng) == 0 {
                    // This is safe; we don't have unicode chars.
                    ch.to_uppercase().next().unwrap()
                } else {
                    ch
                }
            })
            .collect(),
        Mangler::AllUppercase => fragment.to_uppercase(),
        Mangler::ReplaceSeps => fragment
            .chars()
            .map(|ch| {
                let distr_replace = Lazy::new(|| Uniform::new(0, 4));
                if ch == '-' && (*distr_replace).sample(rng) == 0 {
                    const SEPS: &[u8] = b"!_+$";
                    let distr_seps = Lazy::new(|| Uniform::new(0, SEPS.len()));
                    SEPS[(*distr_seps).sample(rng)] as char
                } else {
                    ch
                }
            })
            .collect(),
        Mangler::NoOp => fragment.to_string(),
    }
}

fn get_mangler(rng: &mut dyn RngCore) -> Mangler {
    let distr_one_fourth = Lazy::new(|| Uniform::new(0, 12));
    match (*distr_one_fourth).sample(rng) {
        // 1/4 probability of selecting a mangler
        0 => Mangler::AllUppercase,
        1 => Mangler::RandomUppercase,
        2 => Mangler::ReplaceSeps,
        _ => Mangler::NoOp,
    }
}

fn mangle_fragment(fragment: &str, rng: &mut dyn RngCore) -> String {
    // Select mangling function
    let mangler = get_mangler(rng);
    let new = perform_mangle(fragment, mangler, rng);

    if rng.gen() && mangler != Mangler::NoOp {
        if mangler == Mangler::AllUppercase || mangler == Mangler::RandomUppercase {
            // Don't repeat a case mangling
            return perform_mangle(&new, Mangler::ReplaceSeps, rng);
        } else if mangler == Mangler::ReplaceSeps {
            let mangler = if rng.gen() {
                Mangler::AllUppercase
            } else {
                Mangler::RandomUppercase
            };
            return perform_mangle(&new, mangler, rng);
        }
    }

    new
}

pub(crate) fn shady_filename(rng: &mut dyn RngCore) -> String {
    arr!(const SEPS: [&str; _] = ["-", "!", "_", "+", "$"]);

    // These never change, so no point in regenerating them each time
    let distr_seps = Lazy::new(|| Uniform::new(0, SEPS.len()));
    let distr_nsfw = Lazy::new(|| Uniform::new(0, NSFW.len()));
    let distr_ext = Lazy::new(|| Uniform::new(0, EXT.len()));
    let distr_count = Lazy::new(|| Uniform::new_inclusive(6, 12));

    let hash = generate_hash(rng);

    /* We multiply by 2 so we can put on the separators and suffix
     * The total number of separators plus the suffix works out to be double the nsfw_count.
     * This means the hash_pos must be doubled, too, to be in the right spot.
     */
    let nsfw_count = distr_count.sample(rng) * 2;
    let hash_pos = rng.gen_range(0..nsfw_count) * 2;

    let mut seen_set = HashSet::new();
    (0..nsfw_count)
        .map(|i| {
            if (i & 1) == 1 {
                // This is odd. Do we tack on the suffix or a separator?
                if i == nsfw_count - 1 {
                    EXT[(*distr_ext).sample(rng)].to_string()
                } else {
                    // This position will always be odd, since nsfw_count * 2 is always even.
                    // NB: the range is [0..nsfw_count * 2)
                    //
                    SEPS[(*distr_seps).sample(rng)].to_string()
                }
            } else if i == hash_pos {
                hash.clone()
            } else {
                // Loop until we get a unique NSFW fragment
                loop {
                    let pos = (*distr_nsfw).sample(rng);
                    if seen_set.contains(&pos) {
                        continue;
                    }
                    seen_set.insert(pos);
                    return mangle_fragment(NSFW[pos], rng);
                }
            }
        })
        .collect()
}

arr!(const NSFW: [&str; _] = [
    "0-percent-artificial",
    "0-percent-risk",
    "100-percent-natural",
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
    "all-natural",
    "ambien",
    "america",
    "anal",
    "anal-penetration",
    "anal-sex",
    "anarchist",
    "anarchy",
    "anti-aging-pills",
    "anti-e-pills",
    "anti-estrogen-pills",
    "anti-t-pills",
    "anti-testosterone-pills",
    "antifa",
    "antivirus",
    "apple-giveaway",
    "arpa",
    "ass",
    "assault",
    "ass-beating",
    "audio",
    "babes",
    "bad-mixtape",
    "badonkadonk",
    "bang",
    "bank-transfer",
    "barely-legal",
    "bargains",
    "basic",
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
    "cheap-guns-ammo",
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
    "cock",
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
    "cure",
    "cure-anything",
    "cyber",
    "cyberstalk",
    "cyber-attack",
    "daesh",
    "daesh-meetup",
    "daesh-recruitment",
    "darkweb",
    "dating-4-old-ppl",
    "date-hot-babes",
    "date-hot-chix",
    "date-hot-guys",
    "date-hot-trans-babes",
    "date-hot-trans-chix",
    "date-hot-trans-guys",
    "dead",
    "deals",
    "death",
    "declassified",
    "declassified-dox",
    "deepfake",
    "deepweb",
    "detector",
    "diaper-kink",
    "diazepam",
    "dick",
    "dick-enlargement",
    "diet",
    "diet-supplement",
    "diet-fast-weight-loss",
    "digital",
    "digital-currency",
    "digital-pharmacy",
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
    "easy-men",
    "easy-money",
    "easy-women",
    "echinacea",
    "effexor",
    "election-fraud",
    "email",
    "email-scam",
    "endless-health",
    "endless-money",
    "enhancement",
    "escort",
    "estradiol",
    "estrogen",
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
    "fast",
    "fast-remedy",
    "fast-weight-loss",
    "fight-aging",
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
    "herbal",
    "herbal-remedy",
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
    "homeopathic",
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
    "increase-ur-e",
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
    "jacked-off",
    "jacking",
    "jacking-it",
    "jacking-off",
    "jackpot",
    "jackpot-lottry-winner",
    "jail-beatdown",
    "jail-murder",
    "jail-stabbing",
    "jailbait",
    "jailbreak",
    "jailhouse-beatdown",
    "jailhouse-murder",
    "jailhouse-stabbing",
    "japan",
    "japanese",
    "jar-inserted",
    "jar-jar-porn",
    "java",
    "jelqing",
    "jihad",
    "jizz",
    "jizz-fountain",
    "join-an-orgy",
    "join-now",
    "join-our-cult",
    "join-us",
    "journalist",
    "k9",
    "keygen",
    "keylog",
    "keylogger",
    "king",
    "king-of-coke",
    "king-of-drugs",
    "king-scandal",
    "kinky",
    "kitty",
    "knife-fight",
    "krack",
    "krazy-deals",
    "krazy-good-deal",
    "leak",
    "leaked-documents",
    "leaked-dox",
    "legal",
    "legendary-growth",
    "legend-jackpot",
    "legend-tits",
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
    "loose-slots",
    "loose-sluts",
    "lorazepam",
    "lsd",
    "lotto-winner",
    "m4m",
    "m4t",
    "m4w",
    "macos",
    "maga",
    "magic",
    "magic-cure",
    "magic-mushrooms",
    "magic-weight-loss",
    "make-her-cum",
    "make-money-at-home",
    "make-money-doing-nothing",
    "make-money-fast",
    "male-enhancement",
    "malware",
    "many-babes",
    "marijuana",
    "mature",
    "mdma",
    "medical",
    "medical-magic-mushrooms",
    "medical-mushrooms",
    "medical-mmj",
    "meds",
    "meds-4-cheap",
    "meds-from-canada",
    "meds-from-europe",
    "meds-from-vanuatu",
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
    "more",
    "more-cash",
    "more-income",
    "more-money",
    "more-wins",
    "msm-supplement",
    "multivitamin",
    "mushrooms",
    "muslim",
    "my-tits-are-legend",
    "naked",
    "naked-celebs",
    "nazi-beatdown",
    "new",
    "new-cure",
    "new-drugs",
    "new-income",
    "new-remedy",
    "new-world-order",
    "nft",
    "nigeria-bank-transfer",
    "no-risk",
    "nsfw",
    "nudes",
    "nwo",
    "old-ladies",
    "old-man-gangbang",
    "old-men",
    "old-remedy",
    "online-pharmacy",
    "online-pharma",
    "only-10-dollars",
    "orangutan-sex",
    "organ",
    "organ-selling",
    "overnight",
    "overnight-cure",
    "overnight-growth",
    "overnight-wealth",
    "overnight-weight-loss",
    "overnite-success",
    "overnite-billionaire",
    "overnite-millionaire",
    "overnite-trillionaire",
    "paranomal",
    "password",
    "penetration",
    "penis-enlargement",
    "petit-milf",
    "pewdiepie-sex-tape",
    "pharma",
    "pharmacy",
    "pharma-canada",
    "pharma-euro",
    "pharma-vanuatu",
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
    "porn",
    "prince",
    "prince-scandal",
    "princess",
    "princess-scandal",
    "probe",
    "probing",
    "protein",
    "protein-powder",
    "prozac",
    "psilocybin",
    "psilocybin-online",
    "public",
    "public-sex",
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
    "quickie",
    "quickly-get-rich",
    "quit-ur-job",
    "racket",
    "rapid-growth",
    "rapid-weight-loss",
    "rat-sex",
    "read",
    "real",
    "really-horny-girls",
    "remedy",
    "remote",
    "remote-viewing",
    "reverse",
    "rich",
    "rich-overnight",
    "ripped-fast",
    "ripoff",
    "risperidol",
    "root",
    "rootkit",
    "rope-bondage",
    "rope-porn",
    "russia",
    "russian",
    "russian-bots",
    "scam",
    "scandal",
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
    "sext",
    "sexting",
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
    "shocking",
    "shock-site",
    "shooting",
    "shoplifting",
    "shota",
    "shotacon",
    "slut",
    "sluts",
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
    "t4m",
    "t4t",
    "t4w",
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
    "threesomes-near-u",
    "tighten-my-pussy",
    "tokens",
    "tool",
    "tor",
    "torrent",
    "track-my-ex",
    "track-my-wife",
    "tracker",
    "trans",
    "trans-agenda",
    "trans-doctor",
    "trans-porn",
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
    "vaginal-rejeuvenation",
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
    "w4m",
    "w4t",
    "w4w",
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
]);

arr!(const EXT: [&str; _] = [
    ".app", ".avi", ".bas", ".bat", ".csv", ".divx", ".dll", ".doc", ".docx", ".exe", ".flv", ".gif", ".htm",
    ".html", ".hxt", ".ini", ".jar", ".js", ".jpeg", ".jpg", ".m1v", ".m4a", ".mid", ".midi", ".mkv", ".mod",
    ".mov", ".movie", ".mpa", ".mpe", ".mpeg", ".mpg", ".mp3", ".mp4", ".msi", ".p7r", ".pdf", ".png", ".ppt",
    ".pptx", ".rar", ".sgml", ".snd", ".swf", ".tiff", ".txt", ".webm", ".webp", ".vbs", ".xaf", ".xhtml",
    ".xls", ".xlsx", ".xml", ".zip",
]);
