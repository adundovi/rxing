/*
 * Copyright 2013 ZXing authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::pdf417::pdf_417_common;

/**
 * @author Guenther Grau
 * @author creatale GmbH (christoph.schulz@creatale.de)
 */

// const RATIOS_TABLE :[[f32]] =
//   [[0.0;pdf_417_common::SYMBOL_TABLE.len()];pdf_417_common::BARS_IN_MODULE];

const RATIOS_TABLE: [[f64; pdf_417_common::BARS_IN_MODULE as usize]; 2787] = {
    let mut table =
        [[0.0; pdf_417_common::BARS_IN_MODULE as usize]; pdf_417_common::SYMBOL_TABLE.len()];
    // Pre-computes the symbol ratio table.
    let mut i = 0_usize;
    while i < pdf_417_common::SYMBOL_TABLE.len() {
        // for (int i = 0; i < PDF417Common.SYMBOL_TABLE.length; i++) {
        let mut currentSymbol = pdf_417_common::SYMBOL_TABLE[i];
        let mut currentBit = currentSymbol & 0x1;
        let mut j = 0_usize;
        while j < pdf_417_common::BARS_IN_MODULE as usize {
            // for (int j = 0; j < pdf_417_common::BARS_IN_MODULE; j++) {
            let mut size = 0.0;
            while (currentSymbol & 0x1) == currentBit {
                size += 1.0;
                currentSymbol >>= 1;
            }
            currentBit = currentSymbol & 0x1;
            table[i][pdf_417_common::BARS_IN_MODULE as usize - j - 1] =
                size / pdf_417_common::MODULES_IN_CODEWORD as f64;

            j += 1;
        }

        i += 1;
    }

    table
};

pub fn getDecodedValue(moduleBitCount: &[u32]) -> u32 {
    let decodedValue = getDecodedCodewordValue(&sampleBitCounts(moduleBitCount));
    if decodedValue != -1 {
        return decodedValue as u32;
    }
    getClosestDecodedValue(moduleBitCount) as u32
}

fn sampleBitCounts(moduleBitCount: &[u32]) -> [u32; 8] {
    let bitCountSum: u32 = moduleBitCount.iter().sum(); //MathUtils.sum(moduleBitCount);
    let mut result = [0; pdf_417_common::BARS_IN_MODULE as usize];
    let mut bitCountIndex = 0;
    let mut sumPreviousBits = 0;
    for i in 0..pdf_417_common::MODULES_IN_CODEWORD {
        // for (int i = 0; i < PDF417Common.MODULES_IN_CODEWORD; i++) {
        let sampleIndex: f32 = bitCountSum as f32
            / (2 * pdf_417_common::MODULES_IN_CODEWORD) as f32
            + (i * bitCountSum) as f32 / pdf_417_common::MODULES_IN_CODEWORD as f32;
        if sumPreviousBits as f32 + moduleBitCount[bitCountIndex] as f32 <= sampleIndex {
            sumPreviousBits += moduleBitCount[bitCountIndex];
            bitCountIndex += 1;
        }
        result[bitCountIndex] += 1;
    }
    result
}

fn getDecodedCodewordValue(moduleBitCount: &[u32]) -> i32 {
    let decodedValue = getBitValue(moduleBitCount);
    if pdf_417_common::getCodeword(decodedValue as u32) == -1 {
        -1
    } else {
        decodedValue
    }
}

fn getBitValue(moduleBitCount: &[u32]) -> i32 {
    let mut result: u64 = 0;
    for i in 0..moduleBitCount.len() {
        // for (int i = 0; i < moduleBitCount.length; i++) {
        for _bit in 0..moduleBitCount[i] {
            // for (int bit = 0; bit < moduleBitCount[i]; bit++) {
            result = (result << 1) | (if i % 2 == 0 { 1 } else { 0 });
        }
    }
    result as i32
}

fn getClosestDecodedValue(moduleBitCount: &[u32]) -> i32 {
    let bitCountSum: u32 = moduleBitCount.iter().sum(); //MathUtils.sum(moduleBitCount);
    let mut bitCountRatios = [0.0; pdf_417_common::BARS_IN_MODULE as usize];
    if bitCountSum > 1 {
        for i in 0..bitCountRatios.len() {
            // for (int i = 0; i < bitCountRatios.length; i++) {
            bitCountRatios[i] = moduleBitCount[i] as f64 / bitCountSum as f64;
        }
    }
    let mut bestMatchError = f64::MAX;
    let mut bestMatch = -1_i32;
    for j in 0..RATIOS_TABLE.len() {
        // for (int j = 0; j < RATIOS_TABLE.length; j++) {
        let mut error = 0.0;
        let ratioTableRow = RATIOS_TABLE[j];
        for k in 0..pdf_417_common::BARS_IN_MODULE as usize {
            // for (int k = 0; k < PDF417Common.BARS_IN_MODULE; k++) {
            let diff = ratioTableRow[k] - bitCountRatios[k];
            error += diff * diff;
            if error >= bestMatchError {
                break;
            }
        }
        if error < bestMatchError {
            bestMatchError = error;
            bestMatch = pdf_417_common::SYMBOL_TABLE[j] as i32;
        }
    }
    bestMatch
}
