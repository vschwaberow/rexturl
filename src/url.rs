use std::arch::x86_64::*;
use std::fmt;

#[inline(always)]
fn likely(b: bool) -> bool {
    #[cold]
    fn cold() {}

    if !b {
        cold()
    }
    b
}

#[inline(always)]
fn unlikely(b: bool) -> bool {
    #[cold]
    fn cold() {}

    if b {
        cold()
    }
    b
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    packed: u32,
}

impl Range {
    #[inline(always)]
    pub const fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= 0xFFFF && end <= 0xFFFF, "Range overflow");
        Self {
            packed: ((start as u32) << 16) | (end as u32 & 0xFFFF),
        }
    }

    #[inline(always)]
    pub const fn start(&self) -> usize {
        (self.packed >> 16) as usize
    }

    #[inline(always)]
    pub const fn end(&self) -> usize {
        (self.packed & 0xFFFF) as usize
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.start() >= self.end()
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.end() - self.start()
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Url {
    input: String,
    ranges: [u32; 8],
    flags: u16,
}

const SCHEME_IDX: usize = 0;
const USERNAME_IDX: usize = 1;
const PASSWORD_IDX: usize = 2;
const HOST_IDX: usize = 3;
const PORT_IDX: usize = 4;
const PATH_IDX: usize = 5;
const QUERY_IDX: usize = 6;
const FRAGMENT_IDX: usize = 7;

const HAS_USERNAME: u16 = 1 << 0;
const HAS_PASSWORD: u16 = 1 << 1;
const HAS_PORT: u16 = 1 << 2;
const HAS_QUERY: u16 = 1 << 3;
const HAS_FRAGMENT: u16 = 1 << 4;
const IS_IPV6: u16 = 1 << 5;

#[derive(Debug, Clone, Copy, PartialEq)]

struct CharClass;

impl CharClass {
    const SCHEME_BITS: [u32; 8] = [
        0x03FF_0000,
        0x87FF_FFFE,
        0x07FF_FFFE,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
        0x0000_0000,
    ];

    #[inline(always)]
    fn is_scheme_char(b: u8) -> bool {
        let word_idx = (b >> 5) as usize;
        let bit_idx = b & 31;
        if likely(word_idx < 8) {
            unsafe { (Self::SCHEME_BITS.get_unchecked(word_idx) >> bit_idx) & 1 != 0 }
        } else {
            false
        }
    }

    #[inline(always)]
    fn is_digit(b: u8) -> bool {
        b.wrapping_sub(b'0') <= 9
    }

    #[inline(always)]
    fn is_ascii_alpha(b: u8) -> bool {
        (b | 0x20).wrapping_sub(b'a') <= 25
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn find_byte_simd(haystack: &[u8], needle: u8) -> Option<usize> {
        if haystack.len() < 32 {
            return Self::find_byte_scalar(haystack, needle);
        }

        let needle_vec = _mm256_set1_epi8(needle as i8);
        let mut offset = 0;

        while offset + 32 <= haystack.len() {
            let chunk = _mm256_loadu_si256(haystack.as_ptr().add(offset) as *const __m256i);
            let cmp = _mm256_cmpeq_epi8(chunk, needle_vec);
            let mask = _mm256_movemask_epi8(cmp);

            if mask != 0 {
                return Some(offset + mask.trailing_zeros() as usize);
            }
            offset += 32;
        }

        Self::find_byte_scalar(&haystack[offset..], needle).map(|pos| offset + pos)
    }

    #[inline(always)]
    fn find_byte_scalar(haystack: &[u8], needle: u8) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
    }

    #[inline]
    fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
        if is_x86_feature_detected!("avx2") {
            unsafe { Self::find_byte_simd(haystack, needle) }
        } else {
            Self::find_byte_scalar(haystack, needle)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UrlParseError {
    InvalidScheme,
    InvalidHost,
    InvalidPort,
    InvalidCharacter(char),
    EmptyUrl,
    MalformedUrl,
}

impl fmt::Display for UrlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlParseError::InvalidScheme => write!(f, "Invalid scheme"),
            UrlParseError::InvalidHost => write!(f, "Invalid host"),
            UrlParseError::InvalidPort => write!(f, "Invalid port"),
            UrlParseError::InvalidCharacter(ch) => write!(f, "Invalid character: {ch}"),
            UrlParseError::EmptyUrl => write!(f, "Empty URL"),
            UrlParseError::MalformedUrl => write!(f, "Malformed URL"),
        }
    }
}

impl std::error::Error for UrlParseError {}

#[allow(dead_code)]
impl Url {
    pub fn parse(input: &str) -> Result<Self, UrlParseError> {
        if input.is_empty() {
            return Err(UrlParseError::EmptyUrl);
        }

        let input = input.to_string();
        let mut url = Url {
            input,
            ranges: [0; 8],
            flags: 0,
        };

        url.parse_vectorized()?;
        Ok(url)
    }

    #[inline(always)]
    fn get_range(&self, idx: usize) -> Range {
        Range {
            packed: self.ranges[idx],
        }
    }

    #[inline(always)]
    fn set_range(&mut self, idx: usize, start: usize, end: usize) {
        self.ranges[idx] = ((start as u32) << 16) | (end as u32 & 0xFFFF);
    }

    #[inline(always)]
    fn has_flag(&self, flag: u16) -> bool {
        self.flags & flag != 0
    }

    #[inline(always)]
    fn set_flag(&mut self, flag: u16) {
        self.flags |= flag;
    }

    fn parse_vectorized(&mut self) -> Result<(), UrlParseError> {
        let input_clone = self.input.clone();
        let bytes = input_clone.as_bytes();
        let len = bytes.len();

        if unlikely(len == 0) {
            return Err(UrlParseError::EmptyUrl);
        }

        if len >= 64 {
            unsafe {
                std::arch::x86_64::_mm_prefetch(
                    bytes.as_ptr() as *const i8,
                    std::arch::x86_64::_MM_HINT_T0,
                );
            }
        }

        let mut pos = 0;

        let scheme_end = Self::scan_scheme_optimized(bytes, pos)?;
        self.set_range(SCHEME_IDX, pos, scheme_end);
        pos = scheme_end;

        if unlikely(pos + 2 >= len) || unlikely(&bytes[pos..pos + 3] != b"://") {
            return Err(UrlParseError::MalformedUrl);
        }
        pos += 3;

        if pos + 32 < len && (pos & 63) > 32 {
            unsafe {
                std::arch::x86_64::_mm_prefetch(
                    bytes.as_ptr().add(pos + 32) as *const i8,
                    std::arch::x86_64::_MM_HINT_T0,
                );
            }
        }

        pos = self.parse_authority_hyper_optimized(bytes, pos, len)?;

        self.parse_path_components_bulk(bytes, pos, len)?;

        self.finalize_parsing(len)?;

        Ok(())
    }

    #[inline]
    fn scan_scheme_optimized(bytes: &[u8], start: usize) -> Result<usize, UrlParseError> {
        if unlikely(start >= bytes.len()) || unlikely(!CharClass::is_ascii_alpha(bytes[start])) {
            return Err(UrlParseError::InvalidScheme);
        }

        let mut pos = start + 1;
        while pos + 8 <= bytes.len() {
            let word = unsafe {
                let ptr = bytes.as_ptr().add(pos) as *const u64;
                ptr.read_unaligned()
            };

            let colon_bytes = word ^ 0x3A3A_3A3A_3A3A_3A3A; // Replicate ':' 8 times
            let has_colon = (colon_bytes.wrapping_sub(0x0101_0101_0101_0101))
                & (!colon_bytes)
                & 0x8080_8080_8080_8080;

            if has_colon != 0 {
                for i in 0..8 {
                    if bytes[pos + i] == b':' {
                        return Ok(pos + i);
                    }
                }
            }

            pos += 8;
        }

        while pos < bytes.len() {
            let byte = bytes[pos];
            if byte == b':' {
                return Ok(pos);
            }
            if unlikely(!CharClass::is_scheme_char(byte)) {
                return Err(UrlParseError::InvalidScheme);
            }
            pos += 1;
        }

        Err(UrlParseError::InvalidScheme)
    }

    #[inline]
    fn parse_authority_hyper_optimized(
        &mut self,
        bytes: &[u8],
        start: usize,
        len: usize,
    ) -> Result<usize, UrlParseError> {
        let mut pos = start;

        let authority_end = Self::scan_to_path_query_fragment_simd(bytes, pos, len);

        let at_pos = Self::scan_for_byte_static(bytes, pos, authority_end, b'@');

        if let Some(at_idx) = at_pos {
            let colon_pos = Self::scan_for_byte_static(bytes, pos, at_idx, b':');
            if let Some(colon_idx) = colon_pos {
                self.set_range(USERNAME_IDX, pos, colon_idx);
                self.set_range(PASSWORD_IDX, colon_idx + 1, at_idx);
                self.set_flag(HAS_USERNAME | HAS_PASSWORD);
            } else {
                self.set_range(USERNAME_IDX, pos, at_idx);
                self.set_flag(HAS_USERNAME);
            }
            pos = at_idx + 1;
        }

        pos = self.parse_host_hyper_optimized(bytes, pos, authority_end)?;

        Ok(pos)
    }

    #[inline]
    fn scan_to_path_query_fragment_simd(bytes: &[u8], start: usize, len: usize) -> usize {
        let mut pos = start;

        while pos + 16 <= len {
            let chunk = unsafe {
                let ptr = bytes.as_ptr().add(pos) as *const u128;
                ptr.read_unaligned()
            };

            let slash_mask = Self::create_char_mask_128(chunk, b'/');
            let query_mask = Self::create_char_mask_128(chunk, b'?');
            let fragment_mask = Self::create_char_mask_128(chunk, b'#');

            let combined_mask = slash_mask | query_mask | fragment_mask;

            if combined_mask != 0 {
                for i in 0..16 {
                    if matches!(bytes[pos + i], b'/' | b'?' | b'#') {
                        return pos + i;
                    }
                }
            }
            pos += 16;
        }

        while pos < len && !matches!(bytes[pos], b'/' | b'?' | b'#') {
            pos += 1;
        }
        pos
    }

    #[inline]
    fn create_char_mask_128(word: u128, target: u8) -> u128 {
        let target_repeated = (target as u128) * 0x0101_0101_0101_0101_0101_0101_0101_0101;
        let xor_result = word ^ target_repeated;
        (xor_result.wrapping_sub(0x0101_0101_0101_0101_0101_0101_0101_0101))
            & (!xor_result)
            & 0x8080_8080_8080_8080_8080_8080_8080_8080
    }

    #[inline]
    fn parse_host_hyper_optimized(
        &mut self,
        bytes: &[u8],
        start: usize,
        authority_end: usize,
    ) -> Result<usize, UrlParseError> {
        let mut pos = start;

        if unlikely(pos >= authority_end) {
            return Err(UrlParseError::InvalidHost);
        }

        let host_start = pos;

        if bytes[pos] == b'[' {
            self.set_flag(IS_IPV6);
            pos = Self::scan_ipv6_host_optimized(bytes, pos, authority_end)?;
            self.set_range(HOST_IDX, host_start, pos);

            if pos < authority_end && bytes[pos] == b':' {
                if unlikely(pos + 1 >= authority_end) {
                    return Err(UrlParseError::InvalidPort);
                }
                pos = self.parse_port_optimized_static(bytes, pos + 1, authority_end)?;
            }
        } else {
            let colon_pos = Self::scan_for_byte_static(bytes, pos, authority_end, b':');
            let host_end = colon_pos.unwrap_or(authority_end);

            if unlikely(host_end == pos) {
                return Err(UrlParseError::InvalidHost);
            }

            self.set_range(HOST_IDX, host_start, host_end);
            pos = host_end;

            if let Some(colon_idx) = colon_pos {
                if unlikely(colon_idx + 1 >= authority_end) {
                    return Err(UrlParseError::InvalidPort);
                }
                pos = self.parse_port_optimized_static(bytes, colon_idx + 1, authority_end)?;
            }
        }

        Ok(pos)
    }

    #[inline]
    fn scan_ipv6_host_optimized(
        bytes: &[u8],
        start: usize,
        end: usize,
    ) -> Result<usize, UrlParseError> {
        if let Some(bracket_pos) = CharClass::find_byte(&bytes[start + 1..end], b']') {
            Ok(start + 1 + bracket_pos + 1)
        } else {
            Err(UrlParseError::InvalidHost)
        }
    }

    #[inline]
    fn parse_path_components_bulk(
        &mut self,
        bytes: &[u8],
        start: usize,
        len: usize,
    ) -> Result<(), UrlParseError> {
        let mut pos = start;

        let path_start = pos;
        while pos < len && bytes[pos] != b'?' && bytes[pos] != b'#' {
            pos += 1;
        }
        self.set_range(PATH_IDX, path_start, pos);

        if pos < len && bytes[pos] == b'?' {
            pos += 1;
            let query_start = pos;
            while pos < len && bytes[pos] != b'#' {
                pos += 1;
            }
            if query_start < pos {
                self.set_range(QUERY_IDX, query_start, pos);
                self.set_flag(HAS_QUERY);
            }
        }

        if pos < len && bytes[pos] == b'#' {
            pos += 1;
            if pos < len {
                self.set_range(FRAGMENT_IDX, pos, len);
                self.set_flag(HAS_FRAGMENT);
            }
        }

        Ok(())
    }

    #[inline]
    fn parse_authority_vectorized_static(
        &mut self,
        bytes: &[u8],
        start: usize,
        len: usize,
    ) -> Result<usize, UrlParseError> {
        let mut pos = start;
        let authority_end = Self::scan_to_path_query_fragment_static(bytes, pos, len);

        let at_pos = Self::scan_for_byte_static(bytes, pos, authority_end, b'@');

        if let Some(at_idx) = at_pos {
            let colon_pos = Self::scan_for_byte_static(bytes, pos, at_idx, b':');
            if let Some(colon_idx) = colon_pos {
                self.set_range(USERNAME_IDX, pos, colon_idx);
                self.set_range(PASSWORD_IDX, colon_idx + 1, at_idx);
                self.set_flag(HAS_USERNAME | HAS_PASSWORD);
            } else {
                self.set_range(USERNAME_IDX, pos, at_idx);
                self.set_flag(HAS_USERNAME);
            }
            pos = at_idx + 1;
        }

        pos = self.parse_host_optimized_static(bytes, pos, authority_end)?;

        Ok(pos)
    }

    #[inline]
    fn parse_host_optimized_static(
        &mut self,
        bytes: &[u8],
        start: usize,
        authority_end: usize,
    ) -> Result<usize, UrlParseError> {
        let mut pos = start;

        if pos >= authority_end {
            return Err(UrlParseError::InvalidHost);
        }

        let host_start = pos;

        if bytes[pos] == b'[' {
            self.set_flag(IS_IPV6);
            pos = Self::scan_ipv6_host_static(bytes, pos, authority_end)?;
            self.set_range(HOST_IDX, host_start, pos);

            if pos < authority_end && bytes[pos] == b':' {
                if pos + 1 >= authority_end {
                    return Err(UrlParseError::InvalidPort);
                }
                pos = self.parse_port_optimized_static(bytes, pos + 1, authority_end)?;
            }
        } else {
            let colon_pos = Self::scan_for_byte_static(bytes, pos, authority_end, b':');
            let host_end = colon_pos.unwrap_or(authority_end);

            if host_end == pos {
                return Err(UrlParseError::InvalidHost);
            }

            self.set_range(HOST_IDX, host_start, host_end);
            pos = host_end;

            if let Some(colon_idx) = colon_pos {
                if colon_idx + 1 >= authority_end {
                    return Err(UrlParseError::InvalidPort);
                }
                pos = self.parse_port_optimized_static(bytes, colon_idx + 1, authority_end)?;
            }
        }

        Ok(pos)
    }

    #[inline]
    fn parse_port_optimized_static(
        &mut self,
        bytes: &[u8],
        start: usize,
        end: usize,
    ) -> Result<usize, UrlParseError> {
        if unlikely(start >= end) || unlikely(!CharClass::is_digit(bytes[start])) {
            return Err(UrlParseError::InvalidPort);
        }

        let len = end - start;

        if likely(len <= 5) {
            let port_val = match len {
                1 => (bytes[start] - b'0') as u32,
                2 => {
                    let b0 = (bytes[start] - b'0') as u32;
                    let b1 = (bytes[start + 1] - b'0') as u32;
                    if unlikely(b1 > 9) {
                        return Err(UrlParseError::InvalidPort);
                    }
                    b0 * 10 + b1
                }
                3 => {
                    let b0 = (bytes[start] - b'0') as u32;
                    let b1 = (bytes[start + 1] - b'0') as u32;
                    let b2 = (bytes[start + 2] - b'0') as u32;
                    if unlikely(b1 > 9 || b2 > 9) {
                        return Err(UrlParseError::InvalidPort);
                    }
                    b0 * 100 + b1 * 10 + b2
                }
                4 => {
                    let b0 = (bytes[start] - b'0') as u32;
                    let b1 = (bytes[start + 1] - b'0') as u32;
                    let b2 = (bytes[start + 2] - b'0') as u32;
                    let b3 = (bytes[start + 3] - b'0') as u32;
                    if unlikely(b1 > 9 || b2 > 9 || b3 > 9) {
                        return Err(UrlParseError::InvalidPort);
                    }
                    b0 * 1000 + b1 * 100 + b2 * 10 + b3
                }
                5 => {
                    let b0 = (bytes[start] - b'0') as u32;
                    let b1 = (bytes[start + 1] - b'0') as u32;
                    let b2 = (bytes[start + 2] - b'0') as u32;
                    let b3 = (bytes[start + 3] - b'0') as u32;
                    let b4 = (bytes[start + 4] - b'0') as u32;
                    if unlikely(b1 > 9 || b2 > 9 || b3 > 9 || b4 > 9) {
                        return Err(UrlParseError::InvalidPort);
                    }
                    b0 * 10000 + b1 * 1000 + b2 * 100 + b3 * 10 + b4
                }
                _ => unreachable!(),
            };

            if unlikely(port_val == 0 || port_val > 65535) {
                return Err(UrlParseError::InvalidPort);
            }

            self.set_range(PORT_IDX, start, end);
            self.set_flag(HAS_PORT);
            return Ok(end);
        }

        Err(UrlParseError::InvalidPort)
    }

    #[inline]
    fn parse_path_components_vectorized_static(
        &mut self,
        bytes: &[u8],
        start: usize,
        len: usize,
    ) -> Result<(), UrlParseError> {
        let mut pos = start;

        let path_start = pos;
        while pos < len && bytes[pos] != b'?' && bytes[pos] != b'#' {
            pos += 1;
        }
        self.set_range(PATH_IDX, path_start, pos);

        if pos < len && bytes[pos] == b'?' {
            pos += 1;
            let query_start = pos;
            while pos < len && bytes[pos] != b'#' {
                pos += 1;
            }
            if query_start < pos {
                self.set_range(QUERY_IDX, query_start, pos);
                self.set_flag(HAS_QUERY);
            }
        }

        if pos < len && bytes[pos] == b'#' {
            pos += 1;
            if pos < len {
                self.set_range(FRAGMENT_IDX, pos, len);
                self.set_flag(HAS_FRAGMENT);
            }
        }

        Ok(())
    }

    #[inline]
    fn scan_to_path_query_fragment_static(bytes: &[u8], start: usize, len: usize) -> usize {
        let mut pos = start;
        while pos < len && !matches!(bytes[pos], b'/' | b'?' | b'#') {
            pos += 1;
        }
        pos
    }

    #[inline]
    fn scan_for_byte_static(bytes: &[u8], start: usize, end: usize, target: u8) -> Option<usize> {
        if start >= end || start >= bytes.len() {
            return None;
        }

        let search_slice = &bytes[start..end.min(bytes.len())];
        CharClass::find_byte(search_slice, target).map(|pos| start + pos)
    }

    #[inline]
    fn scan_ipv6_host_static(
        bytes: &[u8],
        start: usize,
        end: usize,
    ) -> Result<usize, UrlParseError> {
        for (offset, &byte) in bytes.iter().enumerate().take(end).skip(start + 1) {
            if byte == b']' {
                return Ok(offset + 1);
            }
        }
        Err(UrlParseError::InvalidHost)
    }

    #[inline]
    fn finalize_parsing(&mut self, len: usize) -> Result<(), UrlParseError> {
        if self.get_range(HOST_IDX).is_empty() {
            return Err(UrlParseError::InvalidHost);
        }

        if self.get_range(PATH_IDX).is_empty() {
            self.set_range(PATH_IDX, len, len);
        }

        Ok(())
    }

    #[inline(always)]
    fn get_component(&self, range: Range) -> &str {
        if range.is_empty() {
            ""
        } else {
            unsafe { self.input.get_unchecked(range.start()..range.end()) }
        }
    }

    #[inline(always)]
    pub fn scheme(&self) -> &str {
        self.get_component(self.get_range(SCHEME_IDX))
    }

    #[inline(always)]
    pub fn username(&self) -> &str {
        if self.has_flag(HAS_USERNAME) {
            self.get_component(self.get_range(USERNAME_IDX))
        } else {
            ""
        }
    }

    #[inline(always)]
    pub fn password(&self) -> &str {
        if self.has_flag(HAS_PASSWORD) {
            self.get_component(self.get_range(PASSWORD_IDX))
        } else {
            ""
        }
    }

    #[inline(always)]
    pub fn host(&self) -> &str {
        self.get_component(self.get_range(HOST_IDX))
    }

    #[inline(always)]
    pub fn host_str(&self) -> Option<&str> {
        let range = self.get_range(HOST_IDX);
        if likely(!range.is_empty()) {
            Some(self.get_component(range))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn port_str(&self) -> Option<&str> {
        if self.has_flag(HAS_PORT) {
            Some(self.get_component(self.get_range(PORT_IDX)))
        } else {
            None
        }
    }

    #[inline]
    pub fn port(&self) -> Option<u16> {
        if !self.has_flag(HAS_PORT) {
            return None;
        }

        let port_str = self.get_component(self.get_range(PORT_IDX));
        let bytes = port_str.as_bytes();

        if bytes.is_empty() {
            return None;
        }

        let mut result = 0u16;
        for &byte in bytes {
            if !CharClass::is_digit(byte) {
                return None;
            }
            let digit = (byte - b'0') as u16;
            if let Some(new_result) = result.checked_mul(10).and_then(|r| r.checked_add(digit)) {
                result = new_result;
            } else {
                return None;
            }
        }

        if result == 0 {
            None
        } else {
            Some(result)
        }
    }

    #[inline(always)]
    pub fn path(&self) -> &str {
        let range = self.get_range(PATH_IDX);
        if likely(!range.is_empty()) {
            self.get_component(range)
        } else {
            "/"
        }
    }

    #[inline(always)]
    pub fn query(&self) -> Option<&str> {
        if self.has_flag(HAS_QUERY) {
            let query_str = self.get_component(self.get_range(QUERY_IDX));
            if likely(!query_str.is_empty()) {
                Some(query_str)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn fragment(&self) -> Option<&str> {
        if self.has_flag(HAS_FRAGMENT) {
            let fragment_str = self.get_component(self.get_range(FRAGMENT_IDX));
            if likely(!fragment_str.is_empty()) {
                Some(fragment_str)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.input
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_url() {
        let url = Url::parse("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.path(), "/");
        assert_eq!(url.query(), None);
        assert_eq!(url.fragment(), None);
    }

    #[test]
    fn test_complex_url() {
        let url =
            Url::parse("https://user:pass@www.example.com:8080/path?query=value#fragment").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.username(), "user");
        assert_eq!(url.password(), "pass");
        assert_eq!(url.host(), "www.example.com");
        assert_eq!(url.port(), Some(8080));
        assert_eq!(url.path(), "/path");
        assert_eq!(url.query(), Some("query=value"));
        assert_eq!(url.fragment(), Some("fragment"));
    }

    #[test]
    fn test_ipv6_url() {
        let url = Url::parse("http://[::1]:8080/").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host(), "[::1]");
        assert_eq!(url.port(), Some(8080));
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn test_url_without_path() {
        let url = Url::parse("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn test_url_with_query_only() {
        let url = Url::parse("https://example.com?query=value").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.path(), "/");
        assert_eq!(url.query(), Some("query=value"));
    }

    #[test]
    fn test_url_with_fragment_only() {
        let url = Url::parse("https://example.com#fragment").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert_eq!(url.path(), "/");
        assert_eq!(url.fragment(), Some("fragment"));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(Url::parse("").is_err());
        assert!(Url::parse("not-a-url").is_err());
        assert!(Url::parse("http://").is_err());
        assert!(Url::parse("http://example.com:").is_err());
        assert!(Url::parse("http://example.com:999999").is_err());
    }

    #[test]
    fn test_edge_cases() {
        let url = Url::parse("https://example.com").unwrap();
        assert_eq!(url.path(), "/");

        let url = Url::parse("https://example.com/").unwrap();
        assert_eq!(url.path(), "/");

        let url = Url::parse("https://example.com?").unwrap();
        assert_eq!(url.query(), None);

        let url = Url::parse("https://example.com#").unwrap();
        assert_eq!(url.fragment(), None);
    }

    #[test]
    fn test_compatibility_methods() {
        let url = Url::parse("https://example.com:8080").unwrap();
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.port_str(), Some("8080"));
        assert_eq!(url.port(), Some(8080));
    }
}
