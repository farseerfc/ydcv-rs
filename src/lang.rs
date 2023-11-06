pub fn is_chinese(s: &str) -> bool {
    for ch in s.chars() {
        if is_chinese_char(ch) {
            return true;
        }
    }

    false
}

#[inline]
fn is_chinese_char(ch: char) -> bool {
    match ch as u32 {
        0x4e00..=0x9fff => true,
        0xff0c => true,            //，
        0x3002 => true,            //。
        0x3400..=0x4dbf => true,   // CJK Unified Ideographs Extension A
        0x20000..=0x2a6df => true, // CJK Unified Ideographs Extension B
        0x2a700..=0x2b73f => true, // CJK Unified Ideographs Extension C
        0x2b740..=0x2b81f => true, // CJK Unified Ideographs Extension D
        0x2b820..=0x2ceaf => true, // CJK Unified Ideographs Extension E
        0x3300..=0x33ff => true,   // https://en.wikipedia.org/wiki/CJK_Compatibility
        0xfe30..=0xfe4f => true,   // https://en.wikipedia.org/wiki/CJK_Compatibility_Forms
        0xf900..=0xfaff => true,   // https://en.wikipedia.org/wiki/CJK_Compatibility_Ideographs
        0x2f800..=0x2fa1f => true, // https://en.wikipedia.org/wiki/CJK_Compatibility_Ideographs_Supplement
        0x00b7 => true,            //·
        0x00d7 => true,            //×
        0x2014 => true,            //—
        0x2018 => true,            //‘
        0x2019 => true,            //’
        0x201c => true,            //“
        0x201d => true,            //”
        0x2026 => true,            //…
        0x3001 => true,            //、
        0x300a => true,            //《
        0x300b => true,            //》
        0x300e => true,            //『
        0x300f => true,            //』
        0x3010 => true,            //【
        0x3011 => true,            //】
        0xff01 => true,            //！
        0xff08 => true,            //（
        0xff09 => true,            //）
        0xff1a => true,            //：
        0xff1f => true,            //？
        _ => false,
    }
}
