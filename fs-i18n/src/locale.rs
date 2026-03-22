// locale.rs — Locale-aware formatting for numbers, floats, dates, and times.
//
// The Language object is the central formatting hub for ALL displayed data.
// Every piece of data — regardless of its source — goes through the Locale
// before being shown to the user.
//
// When the active language changes, all formatting rules change automatically.
//
// Design:
//   Locale    — formatting rules for one locale (separators, date pattern, clock)
//   DateFmt   — how dates are laid out
//   TimeFmt   — 24-hour vs 12-hour clock
//
// Usage:
//   let locale = Locale::for_lang("de");
//   locale.fmt_float(1234.5, 2)  // "1.234,50"
//   locale.fmt_int(1_234_567)    // "1.234.567"
//   locale.fmt_date(2026, 3, 22) // "22.03.2026"
//   locale.fmt_bytes(1536)       // "1,5 KB"

// ── DateFmt ───────────────────────────────────────────────────────────────────

/// How a locale lays out a date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateFmt {
    /// `MM/DD/YYYY` — North American style (`en`)
    MdY,
    /// `DD/MM/YYYY` — European / Latin style (`fr`, `es`, `it`, `pt`)
    DmY,
    /// `DD.MM.YYYY` — Germanic / Eastern-European style (`de`, `nl`, `pl`, ...)
    DotDmY,
    /// `YYYY-MM-DD` — ISO 8601; used as technical default / fallback
    Iso,
    /// `YYYY年MM月DD日` — CJK style (`ja`, `zh`)
    CjkYmd,
    /// `YYYY년 MM월 DD일` — Korean style (`ko`)
    KoreanYmd,
}

// ── TimeFmt ───────────────────────────────────────────────────────────────────

/// Whether a locale uses a 24-hour or 12-hour clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeFmt {
    /// `14:30` — 24-hour clock (most of the world)
    H24,
    /// `2:30 PM` — 12-hour clock with AM/PM (US English)
    H12,
}

// ── Locale ────────────────────────────────────────────────────────────────────

/// Locale-aware formatting rules for one language.
///
/// Returned by [`I18n::locale()`] and the global [`crate::locale()`].
/// Whenever the active language changes the formatter changes with it.
///
/// # Example
///
/// ```rust
/// use fs_i18n::Locale;
///
/// let de = Locale::for_lang("de");
/// assert_eq!(de.fmt_float(1234.5, 2), "1.234,50");
/// assert_eq!(de.fmt_int(1_234_567),   "1.234.567");
/// assert_eq!(de.fmt_date(2026, 3, 22), "22.03.2026");
///
/// let en = Locale::for_lang("en");
/// assert_eq!(en.fmt_float(1234.5, 2), "1,234.50");
/// assert_eq!(en.fmt_date(2026, 3, 22), "03/22/2026");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Locale {
    /// Decimal separator (`.` or `,`).
    pub decimal_sep: char,
    /// Thousands separator (`,`, `.`, `' '`, or `'\0'` = none).
    pub thousands_sep: char,
    /// Date layout style.
    pub date_fmt: DateFmt,
    /// Clock style (24h or 12h).
    pub time_fmt: TimeFmt,
    /// Original BCP-47 language code.
    pub lang: String,
}

impl Locale {
    /// Build the formatting rules for the given BCP-47 language code.
    ///
    /// Only the base language tag is matched (`"de"` matches `"de-AT"`, etc.).
    /// Unknown codes fall back to English / ISO rules.
    pub fn for_lang(lang: &str) -> Self {
        let base = lang.split('-').next().unwrap_or(lang);
        match base {
            // ── Germanic / Eastern-European: decimal comma, dot thousands, DD.MM.YYYY ──
            "de" | "at" | "nl" | "da" | "no" | "nb" | "nn" | "sv" | "fi"
            | "hu" | "pl" | "cs" | "sk" | "hr" | "sl" | "ro" | "bg" | "sr" | "mk"
            | "bs" | "ka" | "az" | "kk" | "ky" | "uz" | "tk" | "mn" | "lv" | "lt"
            | "et" => Self {
                decimal_sep:  ',',
                thousands_sep: '.',
                date_fmt:     DateFmt::DotDmY,
                time_fmt:     TimeFmt::H24,
                lang:         lang.to_string(),
            },

            // ── Romance / other European: decimal comma, dot thousands, DD/MM/YYYY ──
            "fr" | "es" | "it" | "pt" | "ca" | "gl" | "eu" | "oc" | "el"
            | "tr" | "he" | "fa" | "ar" | "ur" => Self {
                decimal_sep:  ',',
                thousands_sep: '.',
                date_fmt:     DateFmt::DmY,
                time_fmt:     TimeFmt::H24,
                lang:         lang.to_string(),
            },

            // ── CJK Japanese / Chinese ────────────────────────────────────────────────
            "ja" | "zh" => Self {
                decimal_sep:  '.',
                thousands_sep: ',',
                date_fmt:     DateFmt::CjkYmd,
                time_fmt:     TimeFmt::H24,
                lang:         lang.to_string(),
            },

            // ── Korean ────────────────────────────────────────────────────────────────
            "ko" => Self {
                decimal_sep:  '.',
                thousands_sep: ',',
                date_fmt:     DateFmt::KoreanYmd,
                time_fmt:     TimeFmt::H24,
                lang:         lang.to_string(),
            },

            // ── English (US): dot decimal, comma thousands, MM/DD/YYYY, 12h ─────────
            "en" => Self {
                decimal_sep:  '.',
                thousands_sep: ',',
                date_fmt:     DateFmt::MdY,
                time_fmt:     TimeFmt::H12,
                lang:         lang.to_string(),
            },

            // ── Everything else: ISO / technical defaults ─────────────────────────────
            _ => Self {
                decimal_sep:  '.',
                thousands_sep: ',',
                date_fmt:     DateFmt::Iso,
                time_fmt:     TimeFmt::H24,
                lang:         lang.to_string(),
            },
        }
    }

    // ── Integer formatting ────────────────────────────────────────────────────

    /// Format a signed integer with locale-appropriate thousands separators.
    ///
    /// ```rust
    /// # use fs_i18n::Locale;
    /// let de = Locale::for_lang("de");
    /// assert_eq!(de.fmt_int(1_234_567), "1.234.567");
    /// assert_eq!(de.fmt_int(-42),       "-42");
    /// ```
    pub fn fmt_int(&self, n: i64) -> String {
        let negative = n < 0;
        let abs      = n.unsigned_abs();
        let grouped  = group_digits(abs, self.thousands_sep);
        if negative { format!("-{grouped}") } else { grouped }
    }

    /// Format an unsigned integer with locale-appropriate thousands separators.
    pub fn fmt_uint(&self, n: u64) -> String {
        group_digits(n, self.thousands_sep)
    }

    // ── Float formatting ──────────────────────────────────────────────────────

    /// Format a float to `decimals` decimal places using locale separators.
    ///
    /// ```rust
    /// # use fs_i18n::Locale;
    /// let de = Locale::for_lang("de");
    /// assert_eq!(de.fmt_float(1234.5,  2), "1.234,50");
    /// assert_eq!(de.fmt_float(0.5,     2), "0,50");
    /// assert_eq!(de.fmt_float(-99.9,   1), "-99,9");
    ///
    /// let en = Locale::for_lang("en");
    /// assert_eq!(en.fmt_float(1234.5,  2), "1,234.50");
    /// ```
    pub fn fmt_float(&self, n: f64, decimals: u8) -> String {
        let negative = n < 0.0;
        let abs      = n.abs();

        // Round to the requested precision before splitting.
        let factor  = 10f64.powi(decimals as i32);
        let rounded = (abs * factor).round() / factor;

        let int_part  = rounded.trunc() as u64;
        let frac_part = ((rounded.fract() * factor).round()) as u64;

        let int_str = group_digits(int_part, self.thousands_sep);

        let result = if decimals == 0 {
            int_str
        } else {
            let frac_str = format!("{:0>width$}", frac_part, width = decimals as usize);
            format!("{int_str}{}{frac_str}", self.decimal_sep)
        };

        if negative { format!("-{result}") } else { result }
    }

    // ── Derived formatters ────────────────────────────────────────────────────

    /// Format a fraction (0.0–1.0) as a percentage with one decimal place.
    ///
    /// ```rust
    /// # use fs_i18n::Locale;
    /// let de = Locale::for_lang("de");
    /// assert_eq!(de.fmt_percent(0.756), "75,6 %");
    /// ```
    pub fn fmt_percent(&self, fraction: f64) -> String {
        format!("{} %", self.fmt_float(fraction * 100.0, 1))
    }

    /// Format a byte count into a human-readable string (B, KB, MB, GB, TB).
    ///
    /// ```rust
    /// # use fs_i18n::Locale;
    /// let de = Locale::for_lang("de");
    /// assert_eq!(de.fmt_bytes(1536),            "1,5 KB");
    /// assert_eq!(de.fmt_bytes(512),             "512 B");
    /// assert_eq!(de.fmt_bytes(1_073_741_824),   "1,0 GB");
    /// ```
    pub fn fmt_bytes(&self, bytes: u64) -> String {
        const KB: u64 = 1_024;
        const MB: u64 = 1_024 * KB;
        const GB: u64 = 1_024 * MB;
        const TB: u64 = 1_024 * GB;

        if bytes >= TB {
            format!("{} TB", self.fmt_float(bytes as f64 / TB as f64, 1))
        } else if bytes >= GB {
            format!("{} GB", self.fmt_float(bytes as f64 / GB as f64, 1))
        } else if bytes >= MB {
            format!("{} MB", self.fmt_float(bytes as f64 / MB as f64, 1))
        } else if bytes >= KB {
            format!("{} KB", self.fmt_float(bytes as f64 / KB as f64, 1))
        } else {
            format!("{} B", bytes)
        }
    }

    // ── Date / time formatting ────────────────────────────────────────────────

    /// Format a date from year / month / day components.
    ///
    /// ```rust
    /// # use fs_i18n::Locale;
    /// assert_eq!(Locale::for_lang("de").fmt_date(2026, 3, 22), "22.03.2026");
    /// assert_eq!(Locale::for_lang("en").fmt_date(2026, 3, 22), "03/22/2026");
    /// assert_eq!(Locale::for_lang("fr").fmt_date(2026, 3, 22), "22/03/2026");
    /// assert_eq!(Locale::for_lang("ja").fmt_date(2026, 3, 22), "2026年03月22日");
    /// ```
    pub fn fmt_date(&self, year: i32, month: u8, day: u8) -> String {
        match self.date_fmt {
            DateFmt::MdY       => format!("{:02}/{:02}/{}", month, day, year),
            DateFmt::DmY       => format!("{:02}/{:02}/{}", day, month, year),
            DateFmt::DotDmY    => format!("{:02}.{:02}.{}", day, month, year),
            DateFmt::Iso       => format!("{}-{:02}-{:02}", year, month, day),
            DateFmt::CjkYmd    => format!("{}年{:02}月{:02}日", year, month, day),
            DateFmt::KoreanYmd => format!("{}년 {:02}월 {:02}일", year, month, day),
        }
    }

    /// Format a time from hour / minute / second components.
    ///
    /// Seconds are currently omitted from the output; they are accepted for
    /// API symmetry and future use.
    ///
    /// ```rust
    /// # use fs_i18n::Locale;
    /// assert_eq!(Locale::for_lang("de").fmt_time(14, 30, 0), "14:30");
    /// assert_eq!(Locale::for_lang("en").fmt_time(14, 30, 0), "2:30 PM");
    /// assert_eq!(Locale::for_lang("en").fmt_time(0,  0,  0), "12:00 AM");
    /// ```
    pub fn fmt_time(&self, hour: u8, minute: u8, _second: u8) -> String {
        match self.time_fmt {
            TimeFmt::H24 => format!("{:02}:{:02}", hour, minute),
            TimeFmt::H12 => {
                let (h12, suffix) = if hour == 0 {
                    (12u8, "AM")
                } else if hour < 12 {
                    (hour, "AM")
                } else if hour == 12 {
                    (12u8, "PM")
                } else {
                    (hour - 12, "PM")
                };
                format!("{}:{:02} {}", h12, minute, suffix)
            }
        }
    }

    /// Format a date+time together.
    pub fn fmt_datetime(&self, year: i32, month: u8, day: u8, hour: u8, minute: u8) -> String {
        format!(
            "{} {}",
            self.fmt_date(year, month, day),
            self.fmt_time(hour, minute, 0)
        )
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Group the decimal digits of `n` with `sep` inserted every 3 digits from the right.
///
/// If `sep == '\0'` or `n < 1000` no separator is inserted.
fn group_digits(n: u64, sep: char) -> String {
    if sep == '\0' || n < 1_000 {
        return n.to_string();
    }

    let s    = n.to_string();
    let len  = s.len();
    let rem  = len % 3; // digits before the first separator
    let mut out = String::with_capacity(len + len / 3);

    for (i, ch) in s.chars().enumerate() {
        if i != 0 && (i % 3 == rem) {
            out.push(sep);
        }
        out.push(ch);
    }
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Integer ──────────────────────────────────────────────────────────────

    #[test]
    fn fmt_int_de() {
        let de = Locale::for_lang("de");
        assert_eq!(de.fmt_int(0),           "0");
        assert_eq!(de.fmt_int(999),         "999");
        assert_eq!(de.fmt_int(1_000),       "1.000");
        assert_eq!(de.fmt_int(1_234_567),   "1.234.567");
        assert_eq!(de.fmt_int(-1_234_567),  "-1.234.567");
    }

    #[test]
    fn fmt_int_en() {
        let en = Locale::for_lang("en");
        assert_eq!(en.fmt_int(1_234_567),  "1,234,567");
        assert_eq!(en.fmt_int(-42),        "-42");
    }

    // ── Float ─────────────────────────────────────────────────────────────────

    #[test]
    fn fmt_float_de() {
        let de = Locale::for_lang("de");
        assert_eq!(de.fmt_float(1234.5,  2), "1.234,50");
        assert_eq!(de.fmt_float(0.5,     2), "0,50");
        assert_eq!(de.fmt_float(-99.9,   1), "-99,9");
        assert_eq!(de.fmt_float(1000.0,  0), "1.000");
    }

    #[test]
    fn fmt_float_en() {
        let en = Locale::for_lang("en");
        assert_eq!(en.fmt_float(1234.5,  2), "1,234.50");
        assert_eq!(en.fmt_float(0.005,   2), "0.01"); // rounds up
    }

    // ── Percent ───────────────────────────────────────────────────────────────

    #[test]
    fn fmt_percent_de() {
        let de = Locale::for_lang("de");
        assert_eq!(de.fmt_percent(0.756), "75,6 %");
        assert_eq!(de.fmt_percent(1.0),   "100,0 %");
    }

    // ── Bytes ─────────────────────────────────────────────────────────────────

    #[test]
    fn fmt_bytes() {
        let de = Locale::for_lang("de");
        assert_eq!(de.fmt_bytes(512),           "512 B");
        assert_eq!(de.fmt_bytes(1536),          "1,5 KB");
        assert_eq!(de.fmt_bytes(1_048_576),     "1,0 MB");
        assert_eq!(de.fmt_bytes(1_073_741_824), "1,0 GB");
    }

    // ── Date ──────────────────────────────────────────────────────────────────

    #[test]
    fn fmt_date_variants() {
        assert_eq!(Locale::for_lang("de").fmt_date(2026, 3, 22), "22.03.2026");
        assert_eq!(Locale::for_lang("en").fmt_date(2026, 3, 22), "03/22/2026");
        assert_eq!(Locale::for_lang("fr").fmt_date(2026, 3, 22), "22/03/2026");
        assert_eq!(Locale::for_lang("ja").fmt_date(2026, 3, 22), "2026年03月22日");
        assert_eq!(Locale::for_lang("ko").fmt_date(2026, 3, 22), "2026년 03월 22일");
        assert_eq!(Locale::for_lang("xx").fmt_date(2026, 3, 22), "2026-03-22");
    }

    // ── Time ──────────────────────────────────────────────────────────────────

    #[test]
    fn fmt_time_24h() {
        let de = Locale::for_lang("de");
        assert_eq!(de.fmt_time(14, 30, 0), "14:30");
        assert_eq!(de.fmt_time(0,  0,  0), "00:00");
    }

    #[test]
    fn fmt_time_12h() {
        let en = Locale::for_lang("en");
        assert_eq!(en.fmt_time(0,  0,  0), "12:00 AM");
        assert_eq!(en.fmt_time(12, 0,  0), "12:00 PM");
        assert_eq!(en.fmt_time(14, 30, 0), "2:30 PM");
        assert_eq!(en.fmt_time(9,  5,  0), "9:05 AM");
    }

    // ── group_digits edge cases ───────────────────────────────────────────────

    #[test]
    fn group_digits_small() {
        assert_eq!(group_digits(0,   ','), "0");
        assert_eq!(group_digits(999, ','), "999");
    }

    #[test]
    fn group_digits_exactly_1000() {
        assert_eq!(group_digits(1_000, ','), "1,000");
    }

    #[test]
    fn group_digits_million() {
        assert_eq!(group_digits(1_000_000, '.'), "1.000.000");
    }
}
