/**
* Base16b family encode / decode
* http://base16b.org/lib/version/0.1/js/base16b.js
* or http://base16b.org/lib/js/base16b.js
**/
/*
Copyright (c) 2009 Base16b.org
Permission is hereby granted, free of charge, to any person
obtaining a copy of this software and associated documentation
files (the "Software"), to deal in the Software without
restriction, including without limitation the rights to use,
copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the
Software is furnished to do so, subject to the following
conditions:
The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.
*/
var Base16b = {
// private variables
_asStart
 : {value : 0x0000,
cp : 0xF0000},
// +UF0000 is the first code point in the Asyntactic script
_noncont : function () {
var nc = []; // array of cp : value mappings for the non-contiguous code points
nc[0] = {value : 0xFFFE, cp : 0xF80A};
nc[1] = {value : 0xFFFF, cp : 0xF80B};
nc[2] = {value : 0x1FFFE, cp : 0xF80C};
nc[3] = {value : 0x1FFFF, cp : 0xF80D};
return nc;
},
// private methods
_CharBytes : function (segmCP) { // return the number of bytes needed for the character. Usually 2.
if (this._fixedCharCodeAt(segmCP,0) && this._fixedCharCodeAt(segmCP,1)) return 2; else return 1;
},
_invertVal : function (segmVal, base) {
// Two's complement of the value for this base
return Math.pow(2, base) - (segmVal +1);
},
_fromCodePoint : function(segmCP, bytes) {
// Map Code Point to a segment value as specified by the mapping table for this base in the Asyntactic script
if (bytes === 2) return this._fixedCharCodeAt(segmCP,0) - this._asStart.cp;
var i;
for (i=0; i < this._noncont().length; i++) {
 // handle non-contiguous code points for last two CPs in bases 16 and 17
if (this._fixedFromCharCode(this._noncont()[i].cp) === segmCP) {
return this._noncont()[i].value;
}
}
},
_toCodePoint : function (segmVal, base) {
// Map a segment value to the Code Point specified by the mapping table for this base in the Asyntactic script
if (base < 16) return this._asStart.cp + segmVal;
var i;
for (i=0; i < this._noncont().length; i++) {
 // handle non-contiguous code points for bases 16 and 17
if (this._noncont()[i].value === segmVal) return this._noncont()[i].cp;
}
return this._asStart.cp + segmVal;
},
24 http://www.opensource.org/licenses/mit-license.php
Base16b.org
under the
license
_fixedFromCharCode : function (codePt) {
// Fix the standard String.FromCharCode method to handle non-BMP unicode planes
// https://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/fromCharCode
if (codePt > 0xFFFF) {
codePt -= 0x10000;
return String.fromCharCode(0xD800 + (codePt >> 10), 0xDC00 + (codePt & 0x3FF));
}
else { return String.fromCharCode(codePt); }
},
_fixedCharCodeAt : function (str, idx) {
// https://developer.mozilla.org/en/Core_JavaScript_1.5_Reference/Global_Objects/String/charCodeAt
var code = str.charCodeAt(idx);
var hi, low;
if (0xD800 <= code && code <= 0xDBFF) { // High surrogate (could change last hex to 0xDB7F to treat high private surrogates as single characters)
hi = code;
low = str.charCodeAt(idx+1);
return ((hi - 0xD800) * 0x400) + (low - 0xDC00) + 0x10000;
}
if (0xDC00 <= code && code <= 0xDFFF) { // Low surrogate
hi = str.charCodeAt(idx-1);
low = code;
return ((hi - 0xD800) * 0x400) + (low - 0xDC00) + 0x10000;
}
return code;
},
// public method for encoding
encode : function (inputArr, base)
/*
Encode an array of pseudo-booleans (0 or 1)
The specification of the encoding is documented elsewhere on this site. (Search Asyntactic script and Base16b.)
*/
{
try {
if (!(base >= 7 && base <= 17)) throw ('invalid encoding base: '+base);
var resultArr
var fullSegments
var remainBits
var segment, bit;
var segmstart;
var segmVal ;
var currsegm;
= [];
= Math.floor(inputArr.length / base);
= inputArr.length - (fullSegments*base);
// construct the value of the bits in the current segment
},
}
catch{
(e)
}
// convert the next segment of base numer of bits to decimal
for (segment=0; segment < fullSegments; segment++) {
 // input and output both read from left to right
segmstart = base*segment;
currsegm = inputArr.slice(segmstart, segmstart + base);
// most significant bit at the start (left) / least significant bit at the end (right).
for (bit = base-1, segmVal = 0; bit >= 0; bit--) {
segmVal += (currsegm[bit] * Math.pow(2,(base-1) - bit));
}
resultArr[segment] = this._fixedFromCharCode(this._toCodePoint (segmVal, base));
}
// encode the termination character
segmVal = 0;
segmstart = base*segment;
currsegm = inputArr.slice(segmstart);
for (bit = remainBits-1; bit >= 0; bit--) {
segmVal += (currsegm[bit] * Math.pow(2,(remainBits-1) - bit));
}
resultArr[segment]= this._fixedFromCharCode(this._toCodePoint(this._invertVal(segmVal, base), base));
return resultArr.join('') ;
alert (e); return false;
// public method for decoding
decode : function (inputStr)
/*
Decode a string encoded in the Asyntactic script. Return an array of pseudo-booleans (0 or 1)
The specification of the encoding is documented elsewhere on this site. (Search Asyntactic script and Base16b.)
*/
{
try {
var resultArr = [];
var termCharBytes = this._CharBytes(inputStr.slice(-2));
var termCharCP = inputStr.slice(-termCharBytes);
 // get the termination character
var termCharVal
 = this._fromCodePoint (termCharCP, termCharBytes);
var bit = 17, base ;
// decode the base from the termination character
while (Math.floor(termCharVal / Math.pow(2, bit-1) ) === 0 && bit >=7 ) {bit--;}
if (!(bit >= 7 && bit <= 17)) throw ('invalid encoding base');
else base = bit;
var segmVal ;
var currCharBytes;
var bytesUsed = 0;
Base16b.org
 license
};
var fullBytes = inputStr.length - termCharBytes;
while (bytesUsed < fullBytes) {
 // decode the code point segments in sequence
currCharBytes = this._CharBytes(inputStr.slice(bytesUsed+2)); // taste before taking a byte
termCharCP = inputStr.slice(bytesUsed, bytesUsed + currCharBytes);
var segmVal = this._fromCodePoint (termCharCP, currCharBytes);
// most significant bit at the start (left) / least significant bit at the end (right).
}
for (bit = (currCharBytes*8)-1; bit >= 0; bit--) {
resultArr.push( Math.floor( (segmVal / Math.pow(2,(bit)))%2));
}
bytesUsed += currCharBytes;
// remainder
var remainVal
= this._invertVal(termCharVal, base);
 // decode the remainder from the termination character
for (bit = (termCharBytes*8)-1; bit >= 0; bit--) {
resultArr.push( Math.floor( (remainVal / Math.pow(2,(bit)))%2));
}
return resultArr;
},
}
catch (e)
{
}
alert (e); return false;
// public method for counting Unicode characters
trueLength : function (inputStr)
/*
Count the number of characters in a string.
This function can handle stings of mixed BMP plane and higher Unicode planes.
Fixes a problem with Javascript which incorrectly that assumes each character is only one byte.
*/
{
var strBytes = inputStr.length;
var strLength
 = 0;
var tallyBytes
 = 0;
try {
while (tallyBytes < strBytes) {
tallyBytes += this._CharBytes(inputStr.slice(tallyBytes, tallyBytes+2));
strLength +=1;
}
return strLength;
}
catch (e)
{
alert (e); return false;
}
}
