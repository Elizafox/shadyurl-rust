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

use once_cell::sync::Lazy;
use rand::{
    distributions::{DistString, Distribution, Uniform},
    prelude::*,
    seq::SliceRandom,
    Rng,
};
use rand_distr::Alphanumeric;

use crate::util::macros::arr;

#[derive(PartialEq, Eq, Copy, Clone)]
enum Mangler {
    NoOp,
    RandomUppercase,
    AllUppercase,
    ReplaceSeps,
    NumberLookalike,
    HeckTransform,
}

fn generate_hash(rng: &mut dyn RngCore) -> String {
    let distr_count = Lazy::new(|| Uniform::new_inclusive(5, 9));
    let count = distr_count.sample(rng);
    Alphanumeric.sample_string(rng, count)
}

fn perform_mangle(rng: &mut dyn RngCore, mangler: Mangler, fragment: &str) -> String {
    match mangler {
        Mangler::RandomUppercase => fragment
            .chars()
            .map(|ch| {
                let distr_cap = Lazy::new(|| Uniform::new(0, 3));
                if (*distr_cap).sample(rng) == 0 {
                    ch.to_uppercase().collect()
                } else {
                    ch.to_string()
                }
            })
            .collect(),
        Mangler::AllUppercase => fragment.to_uppercase(),
        Mangler::ReplaceSeps => fragment
            .chars()
            .map(|ch| {
                let distr_replace = Lazy::new(|| Uniform::new(0, 4));
                if ch == '-' && (*distr_replace).sample(rng) == 0 {
                    arr!(const SEPS: [&str; _] = ["!", "_", "+", "$"]);
                    SEPS.choose(rng).unwrap().to_string()
                } else {
                    ch.to_string()
                }
            })
            .collect(),
        Mangler::NumberLookalike => fragment
            .chars()
            .map(|ch| {
                let distr_replace = Lazy::new(|| Uniform::new(0, 4));
                if (*distr_replace).sample(rng) == 0 {
                    match ch {
                        'o' | 'O' => '0',
                        'a' | 'A' => '4',
                        'e' | 'E' => '3',
                        'g' | 'G' => '9',
                        'i' | 'I' | 'l' | 'L' => '1',
                        's' | 'S' => '5',
                        't' | 'T' => '7',
                        _ => ch,
                    }
                } else {
                    ch
                }
            })
            .collect(),
        Mangler::HeckTransform => {
            let distr_transform = Lazy::new(|| Uniform::new(0, 6));
            match (*distr_transform).sample(rng) {
                0 => heck::AsLowerCamelCase(fragment).to_string(),
                1 => heck::AsUpperCamelCase(fragment).to_string(),
                2 => heck::AsShoutyKebabCase(fragment).to_string(),
                3 => heck::AsShoutySnakeCase(fragment).to_string(),
                4 => heck::AsSnakeCase(fragment).to_string(),
                5 => heck::AsTrainCase(fragment).to_string(),
                _ => unreachable!(),
            }
        }
        Mangler::NoOp => fragment.to_string(),
    }
}

fn get_mangler(rng: &mut dyn RngCore) -> Mangler {
    let distr_mangle = Lazy::new(|| Uniform::new(0, 15));
    match (*distr_mangle).sample(rng) {
        // 1/3 probability of selecting a mangler
        0 => Mangler::AllUppercase,
        1 => Mangler::RandomUppercase,
        2 => Mangler::ReplaceSeps,
        3 => Mangler::NumberLookalike,
        4 => Mangler::HeckTransform,
        _ => Mangler::NoOp,
    }
}

fn mangle_fragment(rng: &mut dyn RngCore, fragment: &str) -> String {
    // Select mangling function
    let distr_second_mangler = Lazy::new(|| Uniform::new(0, 4));
    let mangler = get_mangler(rng);
    let new = perform_mangle(rng, mangler, fragment);

    if (*distr_second_mangler).sample(rng) == 0 {
        // 1/4 chance to apply a second mangler
        let mangler = match mangler {
            Mangler::AllUppercase | Mangler::RandomUppercase => {
                // Don't repeat a case mangling
                if rng.gen() {
                    Mangler::ReplaceSeps
                } else {
                    Mangler::NumberLookalike
                }
            }
            Mangler::ReplaceSeps => match rng.gen_range(0..3) {
                0 => Mangler::AllUppercase,
                1 => Mangler::RandomUppercase,
                2 => Mangler::NumberLookalike,
                _ => unreachable!(),
            },
            Mangler::NumberLookalike => match rng.gen_range(0..3) {
                0 => Mangler::AllUppercase,
                1 => Mangler::RandomUppercase,
                2 => Mangler::ReplaceSeps,
                _ => unreachable!(),
            },
            Mangler::HeckTransform => {
                if rng.gen() {
                    Mangler::ReplaceSeps
                } else {
                    Mangler::NumberLookalike
                }
            }
            Mangler::NoOp => Mangler::NoOp,
        };

        return perform_mangle(rng, mangler, &new);
    }

    new
}

pub(crate) fn shady_filename(rng: &mut dyn RngCore) -> String {
    arr!(const SEPS: [&str; _] = ["!", "_", "+", "~"]);

    // These never change, so no point in regenerating them each time
    let distr_count = Lazy::new(|| Uniform::new_inclusive(3, 6));

    let token_count = distr_count.sample(rng);
    let mut nsfw_str_count = token_count;

    let hash = generate_hash(rng);
    let hash_pos = rng.gen_range(1..token_count);

    let fake_extension_pos = if rng.gen() {
        loop {
            let n = rng.gen_range(1..token_count);
            if n != hash_pos {
                nsfw_str_count -= 1;
                break n;
            }
        }
    } else {
        // Deliberately out of range, so it won't be generated.
        token_count + 1
    };

    // Gather unique strings up front
    let mut nsfw_strs: Vec<_> = NSFW
        .choose_multiple(rng, nsfw_str_count)
        .map(|s| mangle_fragment(rng, s))
        .collect();

    // nsfw strings + extension
    let mut out = Vec::with_capacity(token_count);
    for i in 0..token_count {
        if i > 0 && i != fake_extension_pos {
            // Prepend
            // SAFETY: never fails
            out.push(unsafe { SEPS.choose(rng).unwrap_unchecked().to_string() });
        }

        let push_val = if i == hash_pos {
            hash.clone()
        } else if i == fake_extension_pos {
            // SAFETY: never fails
            unsafe { EXT.choose(rng).unwrap_unchecked().to_string() }
        } else {
            // SAFETY: nsfw_strs always has enough strings
            unsafe { nsfw_strs.pop().unwrap_unchecked() }
        };
        out.push(push_val);
    }

    if rng.gen() || fake_extension_pos <= token_count {
        // Add extension
        // SAFETY: never fails
        out.push(unsafe { EXT_EXE.choose(rng).unwrap_unchecked().to_string() });
    }

    out.into_iter().collect()
}

arr!(const NSFW: [&str; _] = [
    "---click-here---",
    "---install-virus---",
    "0percentartificial",
    "0percentrisk",
    "1$-iphone",
    "1$-android",
    "100percentlegal",
    "100percentnatural",
    "20percentoff",
    "300deadmen",
    "30waystokillyourwife",
    "4chanhacks",
    "419scam",
    "420",
    "500$-cashprize",
    "69",
    "7-11-robbery",
    "800-dollars-4u",
    "9-11-jumper",
    "9-11-video",
    "911-call",
    "abuse",
    "adblock-bypass",
    "admin",
    "ads",
    "advert",
    "affair",
    "aids-bugcatching-parties",
    "al-qaeda",
    "al-qaeda-beheading-vids",
    "al-qaeda-chat",
    "al-qaeda-messageboard",
    "al-qaeda-signup",
    "alien-abduction",
    "all-natural",
    "ambien",
    "anal",
    "anal-penetration",
    "analsex",
    "ancient-cure",
    "ancient-diet-pills",
    "android-unlock",
    "anti-aging-pills",
    "anti-e-pills",
    "anti-estrogen-pills",
    "anti-t-pills",
    "anti-testosterone-pills",
    "antifa-kills-maga",
    "antifamurder",
    "antivirus",
    "apple-giveaway",
    "aryan-brotherhood",
    "aryan-brotherhood-chat",
    "aryan-brotherhood-messageboard",
    "asian",
    "asian-brides",
    "ass-beating",
    "ass2ass",
    "ass2mouth",
    "assault",
    "assfucking",
    "asshole-torn",
    "audio",
    "australian",
    "australian-kangaroo-porn",
    "awesome-real-headshot-vids",
    "azn",
    "babes",
    "back-orifice",
    "backdoor",
    "backyard-accidents-gore",
    "backyard-fireworks-disasters",
    "bad-mixtape",
    "badonkadonk",
    "bang-women",
    "banktransfer",
    "banned-in-the-us",
    "barely-legal",
    "bargains",
    "bbw",
    "bdsm",
    "beatdown",
    "beatingwomen",
    "begin-bank-account-xfer",
    "beheading",
    "best-deals",
    "best-drugs",
    "best-gore",
    "best-pills",
    "bettingonline",
    "bigasses",
    "bigbang",
    "bigbutts",
    "bigcashprize",
    "bigcocks",
    "biggest-cocks",
    "biggest-tits",
    "bigmilf",
    "bigtits",
    "bigwillie",
    "bigwomen",
    "bitcoin-2x",
    "bitcoin-billionaire",
    "bitcoin-cash-paydirt",
    "bitcoin-miner",
    "bitcoin-multiplier",
    "blackmaildox",
    "blood",
    "bloody-murder",
    "bodies",
    "bodybuilders",
    "boko-haram-beheading-vid",
    "bombing",
    "bomb-guide",
    "bomb-instructions",
    "bondage",
    "boyfriendcamera",
    "boyfriendphone",
    "boyfriendtracker",
    "brazilian",
    "brazilian-fart-porn",
    "build-muscle",
    "bupropion",
    "butts",
    "buy-now",
    "buttsex",
    "bypass",
    "cactus-inserted",
    "cactus-sex",
    "calcium",
    "cashmoney",
    "cashnow",
    "casino",
    "casino-loosest-slots",
    "cats-being-beaten",
    "cats-being-eaten",
    "cat-torture",
    "celebaddresses",
    "celebphonenumbers",
    "celebsextape",
    "chatonline",
    "chatwithbabes",
    "cheapcheapcheap",
    "cheap-guns-ammo",
    "cheapcialis",
    "cheapdrugs",
    "cheappills",
    "cheapviagra",
    "cheat-the-system",
    "children-murdered",
    "child-molester-chat",
    "chomo-chat",
    "chomos-online",
    "christian-murder",
    "christian-murders-muslim",
    "chrome-exploiter",
    "chromium-supplement",
    "cialis",
    "classified-doxxx",
    "classmates-of-sex",
    "click",
    "clickme",
    "click-here",
    "clickjack",
    "clone-phone",
    "clone-simcard",
    "clownpenis",
    "cobalt-supplement",
    "cocaine",
    "cockdock",
    "cockfights",
    "coinsonline",
    "coin-multiplier",
    "conspiracy",
    "cookie",
    "cookiestealer",
    "coupons",
    "cowgirl",
    "crackcocaine",
    "crack-bitcoin",
    "crack-facebook",
    "crack-passwords",
    "craigslist-blowjob",
    "craigslist-hookup",
    "crappy-porn",
    "crap-eating-contest",
    "crazy-man-murder",
    "creampie",
    "creditcard",
    "creditcard-numbers",
    "creditscore",
    "crime-tips",
    "crypto",
    "cummy",
    "cum-harder",
    "cuntboys-online",
    "curbstomp-vids",
    "cure-anything",
    "cyberattack",
    "cyberstalk",
    "daesh",
    "daeshmeetup",
    "daeshrecruitment",
    "daeshsignup",
    "dailystormer",
    "darkweb",
    "date-hot-babes",
    "date-hot-chix",
    "date-hot-guys",
    "date-hot-trans-babes",
    "date-hot-trans-chix",
    "date-hot-trans-guys",
    "dating4oldppl",
    "dead-animals",
    "dead-children-pics",
    "dead-people",
    "deals",
    "dealsdealsdeals",
    "death",
    "declassified",
    "declassified-dox",
    "deepfakes",
    "deepweb",
    "deepweb-drugs",
    "deepweb-guns",
    "diaperkink",
    "diazepam",
    "dick2dickdocking",
    "dickenlargement",
    "dickgirls-online",
    "diet-supplement",
    "digital",
    "digitalcurrency",
    "digitalpharmacy",
    "dildoemporium",
    "dmt-online",
    "doctorrecommended",
    "dogfights",
    "dogsex",
    "dogs-being-beaten",
    "dogs-being-eaten",
    "dog-torture",
    "dollarforex",
    "domesticabuse",
    "donate",
    "donkeycock",
    "donkeyshow",
    "dont-just-drizzle",
    "dothisbytomorrow",
    "dothisnow",
    "doxxing",
    "doxxx",
    "drivebyshooting",
    "drugs",
    "dungeon",
    "earn-ur-degree-online",
    "easymen",
    "easymoney",
    "easywomen",
    "echinacea",
    "effexor",
    "electionfraud",
    "emailscam",
    "endless-health",
    "endless-money",
    "enhancement",
    "enter2win",
    "escort",
    "estradiol",
    "estrogen",
    "etherium",
    "etherium-multiplier",
    "euro-forex",
    "evidence",
    "exclusive-sextape",
    "execution",
    "exploit-begin",
    "exploit-install",
    "exploit-start",
    "facebook-blowjob",
    "facebook-hookups",
    "facebook-of-sex",
    "fakelogin",
    "faminepics",
    "fappeningpics",
    "fart-porn",
    "fast-weightloss-diet",
    "fastremedy",
    "fastweightloss",
    "fentanyl",
    "fent-online",
    "fight-aging",
    "finalmoments",
    "finalsolution",
    "fingering",
    "fisting",
    "flashplayer-exploit",
    "flightpoints",
    "footageofdeath",
    "footfetish",
    "footporn",
    "foreign-brides",
    "forex",
    "forex-nobearmarket",
    "forexinterbank",
    "fraudalert",
    "freakout",
    "freeandroid",
    "freeinternet",
    "freeiphone",
    "freemeds",
    "freephone",
    "freepills",
    "freeporn",
    "freeshows",
    "freetv",
    "freevirusremoval",
    "freewebcams",
    "french",
    "friendster-of-sex",
    "frottage",
    "fucktonight",
    "fucking-a-cat",
    "fucking-dog",
    "fucking-raw-chicken",
    "funeral-gone-wrong",
    "gangbang",
    "gaping-asshole",
    "gaping-pussy",
    "gaysex",
    "gay-old-men",
    "gay-orgy",
    "german",
    "german-scat-porn",
    "getagirl",
    "getfuckedtonight",
    "getjacked",
    "getlaid",
    "getlaidtonite",
    "getpersonaldata",
    "getrichovernight",
    "getrichquick",
    "getting-fucked",
    "girlcock",
    "girldick",
    "girlfriendcamera",
    "girlfriendphone",
    "girlfriendtracker",
    "github-of-sex",
    "giveaway",
    "gonesexual",
    "gonewild",
    "gonewrong",
    "gorepix",
    "governmentdocuments",
    "governmentdox",
    "government-fraud",
    "gpstracking",
    "graphicimages",
    "grindrhookup",
    "gruesome-gunshot-wounds",
    "gunfightfootage",
    "gunsonline",
    "hacking-a-facebook",
    "hateminorities",
    "headshot",
    "he-dies",
    "hefuxher",
    "helicoptercrash",
    "herbalremedy",
    "heroin",
    "hijacker",
    "hitler",
    "hitler-sexfilm",
    "holisticmedicine",
    "hookers",
    "hookers-near-u",
    "homeless-man-murder",
    "homeless-woman-murder",
    "homelessdeath",
    "homeopathic",
    "hood-shooting",
    "horny-dads",
    "horny-goat-weed",
    "horny-grandpas",
    "horny-grannies",
    "horny-men",
    "horny-moms",
    "horny-teens",
    "horny-women",
    "horse-sex",
    "horse-slaughter",
    "hotbabes",
    "hotgoats",
    "hotwomen",
    "hotmail",
    "how2printmoney",
    "how2win",
    "how2winatcasino",
    "how-2-skin-a-gerbil",
    "how-to-build-a-bomb",
    "hugecashprize",
    "hugecocks",
    "hypnosis",
    "i-was-probed",
    "ie-exploiter",
    "ied-footage",
    "illegal",
    "illegal-drugs",
    "illegal-guns-4-sale",
    "illegal-porno",
    "iloveu",
    "imake2000aweekathome",
    "impersonateanyone",
    "incestporn",
    "incest-daughter-father",
    "incest-daughter-mother",
    "incest-mother-son",
    "incest-father-son",
    "increase-ur-e",
    "increase-ur-t",
    "install",
    "install-exploit",
    "install-keylogger",
    "install-trojan",
    "install-virus",
    "instant-purchase",
    "interview",
    "intifada",
    "instagram-hack",
    "instagram-of-sex",
    "investmentopportunity",
    "icloud-bypass",
    "icloud-unlock",
    "iphone-unlock",
    "ip-finder",
    "ip-hijacker",
    "ip-stealer",
    "iron-supplements",
    "isis",
    "isisrecruiter",
    "isistrainingcamp",
    "isis-beheading-vid",
    "islam-murder",
    "israel",
    "iwasabductedbyaliens",
    "jackedoff",
    "jackingit",
    "jackingoff",
    "jackpot",
    "jackpot-lottry-winner",
    "jailbait",
    "jailbeatdown",
    "jailbreak",
    "jailhouse-beatdown",
    "jailhouse-murder",
    "jailhouse-stabbing",
    "jailmurder",
    "jailstabbing",
    "japanese",
    "japanese-tentacle-porn",
    "jar-inserted",
    "jar-jar-porn",
    "javascript-exploit",
    "jelqing",
    "jew-nwo",
    "jewish-conspiracy",
    "jewishbanks",
    "jihad",
    "jihadinusa",
    "jizz",
    "jizz-fountain",
    "join-an-orgy",
    "join-now",
    "join-our-cult",
    "join-us",
    "joindaesh",
    "joinisis",
    "jointhearyanbrotherhood",
    "jointhekkk",
    "k9porn",
    "keygen",
    "keylogger",
    "killallimmigrants",
    "killchildren",
    "killgays",
    "killing",
    "killwomen",
    "kingscandal",
    "kinkyporno",
    "kiwifarms-chat",
    "kiwifarms-messageboard",
    "kkk",
    "kkk-chat",
    "kkk-messageboard",
    "kkk-rallies-near-u",
    "knifefight",
    "krackapassword",
    "krazy-deals",
    "krazy-good-deal",
    "leaked-documents",
    "leaked-dox",
    "legalbabes",
    "legend-tits",
    "legendary-growth",
    "legendjackpot",
    "lemonparty",
    "lesbiansfuck",
    "lesbian-gangbang",
    "levitra",
    "lexapro",
    "linkedin-blowjob",
    "linkedin-hookup",
    "linkedin-of-sex",
    "liveleak-for-lesbians",
    "localmen",
    "localwomen",
    "loli-pics",
    "lolicon-pics",
    "lonelywomen",
    "looseslots",
    "loosesluts",
    "lorazepam",
    "lotto-winner",
    "lsdcheap",
    "m4m",
    "m4t",
    "m4w",
    "magic-cure",
    "magicmoney",
    "magicmushrooms",
    "magicweightloss",
    "makehercum",
    "makemoneyathome",
    "makemoneydoingnothing",
    "makemoneyfast",
    "maleenhancement",
    "malware",
    "malwareinstaller",
    "manybabes",
    "marijuana",
    "matureporn",
    "mdmaonline",
    "mdma-cheap",
    "medical-magic-mushrooms",
    "medical-mmj",
    "medicalmushrooms",
    "meds4cheap",
    "medsfromcanada",
    "medsfromchina",
    "medsfromeurope",
    "medsfromvanuatu",
    "meet-men",
    "meet-pedophiles",
    "meet-scat-fetishists",
    "meet-women",
    "megajackpot",
    "megatits",
    "mein-kampf",
    "mercenary",
    "mercenary-kills-animals",
    "mercenary-kills-kids",
    "microsoft-giveaway",
    "mike-pence-gay",
    "mike-pence-naked",
    "milf",
    "milf-tits",
    "military-death",
    "mine-coins-for-free",
    "mom-daughter-fuck",
    "mom-has-sex-with-son",
    "mommy-milkers",
    "mongolian-throat-singing",
    "monster-boners",
    "monster-erection",
    "morecash",
    "moreincome",
    "moremoney",
    "morewins",
    "more-cum",
    "more-sex",
    "msm-supplement",
    "multivitamin",
    "mushrooms",
    "muslim",
    "muslim-murders-christian",
    "myspace-of-sex",
    "my-tits-are-legend",
    "nakedcelebs",
    "naked-guys",
    "naked-ladies",
    "naked-old-men",
    "naked-trans",
    "nazi",
    "nazi-chat",
    "nazi-beatdown",
    "newcure",
    "newdrugs",
    "newincome",
    "newremedy",
    "newworldorder",
    "new-download-site",
    "new-torrent-site",
    "nftscam",
    "nigeria-bank-transfer",
    "no-consent",
    "no-risk",
    "nsfw",
    "nudes",
    "nwo",
    "old-ladies",
    "oldmangangbang",
    "oldmen",
    "oldremedy",
    "one-weird-trick-to-lose-weight",
    "online-horse-race",
    "onlinebetting",
    "onlinepharma",
    "onlinepharmacy",
    "only10dollars",
    "openme",
    "orangutansex",
    "organ-selling",
    "overnight",
    "overnightcure",
    "overnightgrowth",
    "overnightwealth",
    "overnightweightloss",
    "overnitebillionaire",
    "overnitemillionaire",
    "overnitesuccess",
    "overnitetrillionaire",
    "palestinian-bombing",
    "palestinian-murder",
    "passwordhack",
    "peanut-butter-sex",
    "pee-everywhere",
    "pee-sex",
    "pedophilemeet",
    "pedophilesonline",
    "penetration",
    "penisenlargement",
    "petitmilf",
    "pewdiepie-sextape",
    "pharmacanada",
    "pharmachina",
    "pharmacy",
    "pharmaeuro",
    "pharmavanuatu",
    "phishing",
    "phonenumbers",
    "phosphorous-supplements",
    "physical-removal",
    "picsofdeadanimals",
    "picsofdeadpeople",
    "pickup",
    "pickup-girls",
    "pigsex",
    "pills",
    "pills4cheap",
    "pimping-guide",
    "pipebomb",
    "pipebomb-guide",
    "pipebomb-instructions",
    "piratedmovies",
    "piratedmusic",
    "piratedpodcasts",
    "piratedshows",
    "pirate-anything",
    "pirate-games",
    "pirate-movies",
    "pirate-shows",
    "pirate-windows",
    "pizzagate",
    "pokeronline",
    "pokerrealmoney",
    "policebodycam",
    "poop-eating-contest",
    "poop-everywhere",
    "poopsex",
    "popunder",
    "popup",
    "popup-spam",
    "porn",
    "prince-scandal",
    "princess-scandal",
    "probing",
    "protein",
    "proteinpowder",
    "protocols-of-the-elders-of-zion",
    "proud-boys-chat",
    "proud-boys-racism-site",
    "proud-boys-messageboard",
    "prozac",
    "psilocybin",
    "psilocybin-online",
    "publicsex",
    "pussyfuck",
    "putin-naked",
    "putinsdick",
    "qanonreveal",
    "queen-nudes",
    "queenscandal",
    "quickie",
    "quicklygetrich",
    "quiturjob",
    "racism-website",
    "racist",
    "racist-chat",
    "racist-country-music",
    "racist-dubstep",
    "racist-folk-music",
    "racist-grunge",
    "racist-messageboard",
    "racist-raps",
    "racist-rock-music",
    "racket",
    "rapid-growth",
    "rapid-weightloss",
    "ratsex",
    "ready2fuck",
    "ready-for-sex",
    "realcatsex",
    "realdeath",
    "realdogsex",
    "realdonkeyshow",
    "reallyhornygirls",
    "refinance-now",
    "refugee-murder",
    "removevirus",
    "richfast",
    "richovernight",
    "richquick",
    "rickroll",
    "ripoff",
    "rippedfast",
    "risk-free-investment",
    "risperidol",
    "rootkit",
    "ropebondage",
    "ropeporn",
    "russian-bots",
    "russian-brides",
    "scat-porn",
    "school-shooting",
    "scissoring",
    "secretcams",
    "secret-plans",
    "secretary",
    "see-me-naked",
    "seeherwebcam",
    "seemypussy",
    "seemytits",
    "sell-your-organs",
    "seroquel",
    "sexsexsex",
    "sexoffender",
    "sextape",
    "sexting",
    "sextwithgrannies",
    "sextwithgrandpas",
    "sexwithcats",
    "sexwithdogs",
    "sexwithgerbils",
    "sexyladies",
    "sexywomen",
    "sex-slave-auction",
    "shartporn",
    "she-will-never-know",
    "shedies",
    "shefuxhim",
    "shemale",
    "shes-barely-legal",
    "sheswaiting4u",
    "shiteating",
    "shitfountain",
    "shitfucking",
    "shocking",
    "shocksite",
    "shootingpix",
    "shoplifting",
    "shoplifting-tips",
    "shotapics",
    "shotacon-pics",
    "sim-clone",
    "sim-unlock",
    "skinned-alive",
    "skinned-cats",
    "slutcams",
    "sluts",
    "smallcocks",
    "snuff-films",
    "social-security-scam",
    "sodomy",
    "soldering-iron-insertion",
    "sourcecode-leak",
    "spam4u",
    "spambot",
    "spotify-of-sex",
    "spyware",
    "spyonurboyfriend",
    "spyonurgirlfriend",
    "spyonurhusband",
    "spyonurwife",
    "spyware",
    "stalkher",
    "steal",
    "stealfacebookpassword",
    "stealgmailpassword",
    "stealtwitterpassword",
    "steal-bank-password",
    "steal-without-getting-caught",
    "stjohnswort",
    "stolen-android",
    "stolen-iphones",
    "stoned-for-adultery",
    "stoning-video",
    "stumping-vids",
    "subwaydeath",
    "super-boners",
    "super-erection",
    "super-nsfw",
    "supplements",
    "supplementscheap",
    "suppository",
    "sweepstakes",
    "sweepstakes-enter2win",
    "swissbankaccount",
    "swisslottowinner",
    "swissporn",
    "t4m",
    "t4t",
    "t4w",
    "taliban-interview",
    "taliban-meetup",
    "taliban-recruiter",
    "teen",
    "teen-barely-legal",
    "telegram-adult-rooms",
    "terrorist",
    "testosterone",
    "testosterone-supplement",
    "the-truth-about-jews",
    "thetruth",
    "they-hurt-her",
    "theycantstopyou",
    "theyhatethis",
    "threesomes",
    "threesomes-near-u",
    "tighten-my-pussy",
    "tits",
    "titstitstits",
    "tokens",
    "toolbar",
    "toolbar-download",
    "toolbar-install",
    "toohot4tv",
    "toosexy",
    "toosexy4youtube",
    "torrent-anything",
    "torture-manual",
    "torture-photos",
    "torture-videos",
    "totally-legit",
    "totally-legit-hookups",
    "totally-legit-vpns",
    "trackmyex",
    "trackmywife",
    "train-accident-photos",
    "trans-porn",
    "trojaninstaller",
    "trojan-start",
    "turd-fucking",
    "turkey-porn",
    "twitter-blowjob",
    "ugandan-porn-scam",
    "ukranian-brides",
    "unique-investment-opportunity",
    "un-nwo-conspiracy",
    "underground-death",
    "unlocker4anything",
    "ur-hubby-is-cumming",
    "ur-jackpot-awaits",
    "ur-wife-is-cumming",
    "usb-hijacker",
    "vaginapix",
    "vaginal-rejeuvenation",
    "vanadium-supplement",
    "vanuatu-drugs",
    "venereal-disease-pics",
    "viagra",
    "virusinstaller",
    "virus-start",
    "vitamin-b12",
    "vitamin-b6",
    "vitamin-c",
    "vitamin-d",
    "vitamin-e",
    "vitamins",
    "vomitfucking",
    "vomitporn",
    "vpnhacker",
    "vulnexploit",
    "w4m",
    "w4t",
    "w4w",
    "warcrimes",
    "warezinstall",
    "warphotos",
    "watch",
    "watch-her-die",
    "watch-her-get-fukd",
    "watch-her-webcam",
    "watch-him-die",
    "watchmovies4free",
    "watchpeopledie",
    "watchtv4free",
    "web-toolbar",
    "weed",
    "weightloss",
    "wetfartporn",
    "wetpussy",
    "wfh",
    "what-hes-doing",
    "what-shes-doing",
    "whattheydontwantu2know",
    "whitepower",
    "whos-my-husband-txting",
    "whos-my-wife-txting",
    "wifepussy",
    "winatblackjack",
    "winatpoker",
    "winbig",
    "winner-click-here",
    "winning-lotto-numbers",
    "woman-gets-stoned",
    "womanbeaten",
    "workathome",
    "worldstar",
    "worldstarfight",
    "worm",
    "worminstall",
    "xtc-cheap",
    "xxx-porn",
    "xxx-pussy",
    "xxx-tits",
    "xxxtra",
    "xylophone-sex",
    "yenforex",
    "you-won",
    "youre-a-winner",
    "yourip",
    "yourssn",
    "youtube-download",
    "zinc-supplement",
    "zebra-porn",
    "zipper-open-pics",
    "zoo",
    "zooporn",
]);

arr!(const EXT: [&str; _] = [
    ".avi", ".bas", ".csv", ".divx", ".dll", ".doc", ".docx", ".flv", ".gif", ".htm", ".html",
    ".ini", ".jar", ".js", ".jpeg", ".jpg", ".m1v", ".m4a", ".mid", ".midi", ".mkv", ".mod", ".mov",
    ".movie", ".mpa", ".mpe", ".mpeg", ".mpg", ".mp3", ".mp4", ".p7r", ".pdf", ".png", ".ppt",
    ".pptx", ".rar", ".sgml", ".snd", ".swf", ".tiff", ".txt", ".webm", ".webp", ".vbs", ".xaf",
    ".xhtml", ".xls", ".xlsx", ".xml", ".zip",
]);

arr!(const EXT_EXE: [&str; _] = [
    ".app", ".bat", ".dmg", ".exe", ".msi", ".run", ".script",
]);
