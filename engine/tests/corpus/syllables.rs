//! K. Real Vietnamese syllables (telex) — the long-term regression corpus.

use super::{case, TestCase};

pub const CASES: &[TestCase] = &[
    case!(['t', 'h', 'ủ', 'y'], "thủy"),
    case!(['h', 'ủ', 'y'], "hủy"),
    case!(['k', 'h', 'u', 'y', 'u', 'r'], "khuỷu"), // kh + u + y + u + hook → tone on middle y
    case!(['t', 'h', 'u', 'y', 'r'], "thủy"),
    case!(['t', 'h', 'u', 'o', 'w', 'r'], "thuở"), // u,o,w then tone: keeps [U,OHorn]; tone on ơ
    case!(['c', 'h', 'u', 'y', 'ệ', 'n'], "chuyện"),
    case!(['t', 'h', 'u', 'y', 'ê', 'n'], "thuyên"),
    case!(['n', 'g', 'ư', 'ơ', 'i', 'f'], "người"),
    case!(['n', 'g', 'u', 'o', 'w', 'i', 'f'], "người"),
    case!(['m', 'u', 'ố', 'n'], "muốn"),
    case!(['q', 'u', 'á'], "quá"),
    case!(['q', 'u', 'o', 'o', 'c', 's'], "quốc"),
    case!(['n', 'u', 'o', 'w', 'c', 's'], "nước"),
    case!(['t', 'i', 'e', 'e', 'n', 'f'], "tiền"),
    case!(['c', 'ư', 'ờ', 'i'], "cười"),
    case!(['n', 'g', 'h', 'i', 'a', 'f'], "nghìa"),
    case!(['g', 'i', 'ờ'], "giờ"),
    case!(['c', 'h', 'à', 'n'], "chàn"),
    case!(['a', 'n', 'h'], "anh"),
    case!(['e', 'm'], "em"),
    case!(['ơ', 'n'], "ơn"),
    case!(['ô', 'n', 'g'], "ông"),
    case!(['s', 'ư', 'a', 'r'], "sửa"),
    case!(['m', 'ư', 'a', 's'], "mứa"),
    case!(['q', 'u', 'y'], "quy"),
    case!(['q', 'u', 'y', 's'], "quý"),
    case!(['t', 'o', 'a', 'n', 's'], "toán"),
    case!(['t', 'o', 'a', 'f'], "toà"),
    case!(['h', 'o', 'a', 'j'], "hoạ"),
    case!(['a', 'o', 's'], "áo"),
    case!(['y', 'e', 'e', 'u', 's'], "yếu"),
    // y as zero onset in common closed syllables
    case!(['y', 'ê', 'u'], "yêu"),
    case!(['y', 'ê', 'u', 'f'], "yều"),
    // nh onset
    case!(['n', 'h', 'a'], "nha"),
    case!(['n', 'h', 'a', 'f'], "nhà"),
    case!(['n', 'h', 'a', 'r'], "nhả"),
    case!(['n', 'h', 'a', 'j'], "nhạ"),
    case!(['n', 'h', 'i', 'ê', 'u', 'f'], "nhiều"),
    case!(['n', 'h', 'ậ', 't'], "nhật"),
    case!(['n', 'h', 'a', 'n', 'h', 's'], "nhánh"),
];