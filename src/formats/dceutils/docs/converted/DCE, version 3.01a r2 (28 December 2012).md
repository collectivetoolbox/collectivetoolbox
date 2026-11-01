# DCE, version 3.01a r2 (28 December 2012)

## DCE 3.01a

```csv
"DCE, version 3.01a r2 (28 December 2012)
Note that this is a minor revision of version 3.01a, and does not add any entities or change any mappings.
"

The system specified herein is an information representation
"system and binary file format. Most DCE entities are represented as single 8-bit bytes, and the mappings of byte to"
entity are determined by switching entities for an effectively unlimited set of entities. Some mapping sheets may use more
bytes per entity in future versions.

The required structure of a file is as follows:

First four bytes should be: 44434565 (ASCII DCEe)
Fifth byte should be: 02 (Begin DCE content)
Sixth byte should be: 01 (DCE version selector byte)
"Seventh to n bytes should be: DCE version mapping bytes, until a byte FD is reached, which indicates a complete"
drop back to the core mapping sheet.
Next bytes should be: Document content
Next to last byte should be: FD (full drop back)
Final byte should be: 03 (end DCE content)
Example document (Hello World!): 44 43 45 65 02 01 02 FD 80 48 65 6C 6C 6F 20 57 6F 72 6C 64 21 81 FD 03


Notes about this specification:

"1) Mapping sheets always map byte 00 to DCE Character (Dc) 0, “Null”. This is an immutable mapping."
"2) Mapping sheets always map byte FD to Dc 13, “Full drop back”. This is an immutable mapping."
"3) Mapping sheets always map byte FE to Dc 14, “Drop back”. This is an immutable mapping."
"4) Mapping sheets always leave byte FF unmapped, for processing use. This is an immutable mapping."
5) The printable ASCII characters and some other ASCII characters are mapped in Core at their ASCII byte values for compatibility.

Definitions:

Boundings: Document structure information to provide a context for one or more DCE strings
Immutable: A flag on a mapping that indicates that this assignment will not be changed in future versions
of the standard. This is needed for characters necessary to the correct functioning of the
standard with other revisions.
Unbounded: Indicates that a given DCE string has no boundings.
```

## DCE Character List by ID

```csv
Dc ID or ID Sequence,Unicode mapping,Dc Decomposition,Dc Name,Dc Class,Character or representation,Mappings,Description and specification,Variants,Annotations,Other names,See also,Internal ASCII representation (deprecated),,Mojikyo mapping,TRON mapping
0,0,,Null,Technical:Controls,␀,[All sheets: 00]
1,40,,At,General:Symbols,@,Core:40,@. Needed in CDCE prior to main DCE specification for syntactical purposes.
2,,,Begin separator,Semantic:Syntactical,(begin separator),Semantic Records:01,Begin a DCE separator string.
3,,,End separator,Semantic:Syntactical,(end separator),Semantic Records:02,End a DCE separator string.
4,,,Range,Semantic:Syntactical,to ,Semantic Records:03,The DCE range character. Used to represent ranges in DCE. See the wiki for more information.,,,,,\>
5,,,Record summation,Semantic:Syntactical,and ,Semantic Records:04,Used to denote the union of two records. See the wiki for more information.,,,,,=
6,,,Begin number,Semantic:Syntactical,(begin number),Semantic Records:05,Begin a DCE number.
7,,,End number,Semantic:Syntactical,(end number),Semantic Records:06,End a DCE number.
8,,,Unknown value,Semantic:Substitution,#,Semantic Records:07,Substitution character for an unknown value.,,,,,#
9,,,Gregorian calendar identifier,Semantic:Syntactical,Gregorian calendar ,Semantic Records:08,Internal DCE calendar identifier for the Gregorian calendar.,,,,,G
10,,,Positive,Semantic:Syntactical,+,Semantic Records:09,Positive value (syntactical representation — This is not the + character),,,,,+
11,,,Negative,Semantic:Syntactical,-,Semantic Records:0A,Negative (syntactical representation — This is not the − character),,,,,-
12,2E,,Decimal point,General:Mathematics,",",Mathematics:01,,,,,,.
13,,,Full drop back,Technical:Internal,(full drop back),[All sheets: FD],Drop back to the core sheet
14,,,Drop back,Technical:Internal,(drop back),[All sheets: FE],Drop back to the enclosing sheet
15,,,Begin DCE content,Technical:Internal,(begin file),Core:02,Begins a DCE region
16,,,DCE Version Selectors,Sheet:Top,,Core:01
17,,,No mapping,Technical:Internal,,[All sheets: FF],Processing use only (not for interchange)
18,20,,Word separator,General:Punctuation,,Core:20
19,21,,Exclamation,General:Punctuation,!,Core:21
20,22,,Double quotation mark,General:Punctuation,"""",Core:22
21,23,,Number sign,General:Number,#,Core:23,,,,"octothorpe, hash, crosshatch"
22,24,,Dollar,General:Financial,$,Core:24
23,25,,Percent,General:Symbols,%,Core:25
24,26,,And,General:Symbols,&,Core:26
25,27,,Apostrophe,General:Punctuation,',Core:27
26,28,,Left parenthesis,General:Punctuation,(,Core:28,,,,left bracket
27,29,,Right parenthesis,General:Punctuation,),Core:29,,,,right bracket
28,2A,,Asterisk,General:Symbols,*,Core:2A,,,,star
29,2B,,Plus,General:Mathematics,+,Core:2B
30,2C,,Comma,General:Punctuation,",",Core:2C
31,2010,,Hyphen,General:Punctuation,‐,Core:2D
32,2E,,Full stop,General:Punctuation,.,Core:2E,,,,period
33,2F,,Solidus,General:Punctuation,/,Core:2F,,,,"virgule, stroke, slash"
34,30,,Zero,General:Mathematics,0,Core:30
35,31,,One,General:Mathematics,1,Core:31
36,32,,Two,General:Mathematics,2,Core:32
37,33,,Three,General:Mathematics,3,Core:33
38,34,,Four,General:Mathematics,4,Core:34
39,35,,Five,General:Mathematics,5,Core:35
40,36,,Six,General:Mathematics,6,Core:36
41,37,,Seven,General:Mathematics,7,Core:37
42,38,,Eight,General:Mathematics,8,Core:38
43,39,,Nine,General:Mathematics,9,Core:39
44,3A,,Colon,General:Punctuation,:,Core:3A
45,3B,,Semicolon,General:Punctuation,;,Core:3B
46,3C,,Less than,General:Mathematics,<,Core:3C
47,3D,,Equals,General:Mathematics,=,Core:3D
48,3E,,Greater than,General:Mathematics,>,Core:3E
49,3F,,Question,General:Punctuation,?,Core:3F
50,41,,Latin Capital Letter A,Script:Latin,A,Core:41
51,42,,Latin Capital Letter B,Script:Latin,B,Core:42
52,43,,Latin Capital Letter C,Script:Latin,C,Core:43
53,44,,Latin Capital Letter D,Script:Latin,D,Core:44
54,45,,Latin Capital Letter E,Script:Latin,E,Core:45
55,46,,Latin Capital Letter F,Script:Latin,F,Core:46
56,47,,Latin Capital Letter G,Script:Latin,G,Core:47
57,48,,Latin Capital Letter H,Script:Latin,H,Core:48
58,49,,Latin Capital Letter I,Script:Latin,I,Core:49
59,4A,,Latin Capital Letter J,Script:Latin,J,Core:4A
60,4B,,Latin Capital Letter K,Script:Latin,K,Core:4B
61,4C,,Latin Capital Letter L,Script:Latin,L,Core:4C
62,4D,,Latin Capital Letter M,Script:Latin,M,Core:4D
63,4E,,Latin Capital Letter N,Script:Latin,N,Core:4E
64,4F,,Latin Capital Letter O,Script:Latin,O,Core:4F
65,50,,Latin Capital Letter P,Script:Latin,P,Core:50
66,51,,Latin Capital Letter Q,Script:Latin,Q,Core:51
67,52,,Latin Capital Letter R,Script:Latin,R,Core:52
68,53,,Latin Capital Letter S,Script:Latin,S,Core:53
69,54,,Latin Capital Letter T,Script:Latin,T,Core:54
70,55,,Latin Capital Letter U,Script:Latin,U,Core:55
71,56,,Latin Capital Letter V,Script:Latin,V,Core:56
72,57,,Latin Capital Letter W,Script:Latin,W,Core:57
73,58,,Latin Capital Letter X,Script:Latin,X,Core:58
74,59,,Latin Capital Letter Y,Script:Latin,Y,Core:59
75,5A,,Latin Capital Letter Z,Script:Latin,Z,Core:5A
76,5B,,Left bracket,General:Punctuation,[,Core:5B,,,,left square bracket
77,5C,,Reversed solidus,General:Punctuation,\,Core:5C,,,,backslash
78,5D,,Right bracket,General:Punctuation,],Core:5D,,,,right square bracket
79,5E,,Circumflex punctuation,General:Punctuation,^,Core:5E
80,5F,,Underscore,General:Punctuation,_,Core:5F,,,,low line
81,60,,Grave punctuation,General:Punctuation,`,Core:60
82,61,,Latin Small Letter A,Script:Latin,a,Core:61
83,62,,Latin Small Letter B,Script:Latin,b,Core:62
84,63,,Latin Small Letter C,Script:Latin,c,Core:63
85,64,,Latin Small Letter D,Script:Latin,d,Core:64
86,65,,Latin Small Letter E,Script:Latin,e,Core:65
87,66,,Latin Small Letter F,Script:Latin,f,Core:66
88,67,,Latin Small Letter G,Script:Latin,g,Core:67
89,68,,Latin Small Letter H,Script:Latin,h,Core:68
90,69,,Latin Small Letter I,Script:Latin,I,Core:69
91,6A,,Latin Small Letter J,Script:Latin,j,Core:6A
92,6B,,Latin Small Letter K,Script:Latin,k,Core:6B
93,6C,,Latin Small Letter L,Script:Latin,l,Core:6C
94,6D,,Latin Small Letter M,Script:Latin,m,Core:6D
95,6E,,Latin Small Letter N,Script:Latin,n,Core:6E
96,6F,,Latin Small Letter O,Script:Latin,o,Core:6F
97,70,,Latin Small Letter P,Script:Latin,p,Core:70
98,71,,Latin Small Letter Q,Script:Latin,q,Core:71
99,72,,Latin Small Letter R,Script:Latin,r,Core:72
100,73,,Latin Small Letter S,Script:Latin,s,Core:73
101,74,,Latin Small Letter T,Script:Latin,t,Core:74
102,75,,Latin Small Letter U,Script:Latin,u,Core:75
103,76,,Latin Small Letter V,Script:Latin,v,Core:76
104,77,,Latin Small Letter W,Script:Latin,w,Core:77
105,78,,Latin Small Letter X,Script:Latin,x,Core:78
106,79,,Latin Small Letter Y,Script:Latin,y,Core:79
107,7A,,Latin Small Letter Z,Script:Latin,z,Core:7A
108,7B,,Left brace,General:Punctuation,{,Core:7B,,,,left curly bracket
109,7C,,Vertical line,General:Punctuation,|,Core:7C,,,,"vertical bar, pipe"
110,7D,,Right brace,General:Punctuation,},Core:7D,,,,right curly bracket
111,7E,,Tilde punctuation,General:Punctuation,~,Core:7E
112,,,DCE Version Selector 3.0a,Technical:Version,,DCE Version Selectors:01
113,,,End DCE content,Technical:Internal,(end file),Core:01,Ends a DCE region
114,,,Begin unbounded content,Semantic:Structure,,Core:80,Begins an unbounded DCE string
115,,,End unbounded content,Semantic:Structure,,Core:81,Ends an unbounded DCE string
116,8,,Backspace,Technical:Controls,,Core:08
117,9,,Horizontal tabulation,General:Whitespace,,Core:09,,,,"character tabulation, tab"
118,B,,Vertical tabulation,General:Whitespace,,Core:0B,,,,line tabulation
119,A,,New line,General:Whitespace,,Punctuation and Whitespace:01,,"120,121",,"line feed, end of line"
120,A,"122,119",Line feed,General:Whitespace,,Core:0A,,,,"new line, end of line"
121,D,"123,119",Carriage return,General:Whitespace,,Core:0D
122,FE00,,Variant Selector 1,Technical:VS,︀,Variant Selectors:01
123,FE01,,Variant Selector 2,Technical:VS,︁,Variant Selectors:02
124,FE02,,Variant Selector 3,Technical:VS,︂,Variant Selectors:03
125,FE03,,Variant Selector 4,Technical:VS,︃,Variant Selectors:04
126,FE04,,Variant Selector 5,Technical:VS,︄,Variant Selectors:05
127,,,Base 64 encapsulation digit 0,Technical:Encapsulation,,Core:82
128,,,Base 64 encapsulation digit 1,Technical:Encapsulation,,Core:83
129,,,Base 64 encapsulation digit 2,Technical:Encapsulation,,Core:84
130,,,Base 64 encapsulation digit 3,Technical:Encapsulation,,Core:85
131,,,Base 64 encapsulation digit 4,Technical:Encapsulation,,Core:86
132,,,Base 64 encapsulation digit 5,Technical:Encapsulation,,Core:87
133,,,Base 64 encapsulation digit 6,Technical:Encapsulation,,Core:88
134,,,Base 64 encapsulation digit 7,Technical:Encapsulation,,Core:89
135,,,Base 64 encapsulation digit 8,Technical:Encapsulation,,Core:8A
136,,,Base 64 encapsulation digit 9,Technical:Encapsulation,,Core:8B
137,,,Base 64 encapsulation digit 10,Technical:Encapsulation,,Core:8C
138,,,Base 64 encapsulation digit 11,Technical:Encapsulation,,Core:8D
139,,,Base 64 encapsulation digit 12,Technical:Encapsulation,,Core:8E
140,,,Base 64 encapsulation digit 13,Technical:Encapsulation,,Core:8F
141,,,Base 64 encapsulation digit 14,Technical:Encapsulation,,Core:90
142,,,Base 64 encapsulation digit 15,Technical:Encapsulation,,Core:91
143,,,Base 64 encapsulation digit 16,Technical:Encapsulation,,Core:92
144,,,Base 64 encapsulation digit 17,Technical:Encapsulation,,Core:93
145,,,Base 64 encapsulation digit 18,Technical:Encapsulation,,Core:94
146,,,Base 64 encapsulation digit 19,Technical:Encapsulation,,Core:95
147,,,Base 64 encapsulation digit 20,Technical:Encapsulation,,Core:96
148,,,Base 64 encapsulation digit 21,Technical:Encapsulation,,Core:97
149,,,Base 64 encapsulation digit 22,Technical:Encapsulation,,Core:98
150,,,Base 64 encapsulation digit 23,Technical:Encapsulation,,Core:99
151,,,Base 64 encapsulation digit 24,Technical:Encapsulation,,Core:9A
152,,,Base 64 encapsulation digit 25,Technical:Encapsulation,,Core:9B
153,,,Base 64 encapsulation digit 26,Technical:Encapsulation,,Core:9C
154,,,Base 64 encapsulation digit 27,Technical:Encapsulation,,Core:9D
155,,,Base 64 encapsulation digit 28,Technical:Encapsulation,,Core:9E
156,,,Base 64 encapsulation digit 29,Technical:Encapsulation,,Core:9F
157,,,Base 64 encapsulation digit 30,Technical:Encapsulation,,Core:A0
158,,,Base 64 encapsulation digit 31,Technical:Encapsulation,,Core:A1
159,,,Base 64 encapsulation digit 32,Technical:Encapsulation,,Core:A2
160,,,Base 64 encapsulation digit 33,Technical:Encapsulation,,Core:A3
161,,,Base 64 encapsulation digit 34,Technical:Encapsulation,,Core:A4
162,,,Base 64 encapsulation digit 35,Technical:Encapsulation,,Core:A5
163,,,Base 64 encapsulation digit 36,Technical:Encapsulation,,Core:A6
164,,,Base 64 encapsulation digit 37,Technical:Encapsulation,,Core:A7
165,,,Base 64 encapsulation digit 38,Technical:Encapsulation,,Core:A8
166,,,Base 64 encapsulation digit 39,Technical:Encapsulation,,Core:A9
167,,,Base 64 encapsulation digit 40,Technical:Encapsulation,,Core:AA
168,,,Base 64 encapsulation digit 41,Technical:Encapsulation,,Core:AB
169,,,Base 64 encapsulation digit 42,Technical:Encapsulation,,Core:AC
170,,,Base 64 encapsulation digit 43,Technical:Encapsulation,,Core:AD
171,,,Base 64 encapsulation digit 44,Technical:Encapsulation,,Core:AE
172,,,Base 64 encapsulation digit 45,Technical:Encapsulation,,Core:AF
173,,,Base 64 encapsulation digit 46,Technical:Encapsulation,,Core:B0
174,,,Base 64 encapsulation digit 47,Technical:Encapsulation,,Core:B1
175,,,Base 64 encapsulation digit 48,Technical:Encapsulation,,Core:B2
176,,,Base 64 encapsulation digit 49,Technical:Encapsulation,,Core:B3
177,,,Base 64 encapsulation digit 50,Technical:Encapsulation,,Core:B4
178,,,Base 64 encapsulation digit 51,Technical:Encapsulation,,Core:B5
179,,,Base 64 encapsulation digit 52,Technical:Encapsulation,,Core:B6
180,,,Base 64 encapsulation digit 53,Technical:Encapsulation,,Core:B7
181,,,Base 64 encapsulation digit 54,Technical:Encapsulation,,Core:B8
182,,,Base 64 encapsulation digit 55,Technical:Encapsulation,,Core:B9
183,,,Base 64 encapsulation digit 56,Technical:Encapsulation,,Core:BA
184,,,Base 64 encapsulation digit 57,Technical:Encapsulation,,Core:BB
185,,,Base 64 encapsulation digit 58,Technical:Encapsulation,,Core:BC
186,,,Base 64 encapsulation digit 59,Technical:Encapsulation,,Core:BD
187,,,Base 64 encapsulation digit 60,Technical:Encapsulation,,Core:BE
188,,,Base 64 encapsulation digit 61,Technical:Encapsulation,,Core:BF
189,,,Base 64 encapsulation digit 62,Technical:Encapsulation,,Core:C0
190,,,Base 64 encapsulation digit 63,Technical:Encapsulation,,Core:C1
191,,,Begin encapsulated UTF-8,Technical:Encapsulation,,Core:C3,Begin Base 64 encapsulated UTF-8
192,,,End encapsulated UTF-8,Technical:Encapsulation,,Core:C4,End Base 64 encapsulated UTF-8
193,C,,Form feed,General:Whitespace,,Core:0C
194,,,DCE Version Selector 3.01a,Technical:Version,,DCE Version Selectors:02
195,,,Base 64 encapsulation padding character,Technical:Encapsulation,,Core:C2
196,,,Variant Selectors,Sheet:Top,,Core:C5
197,,,Semantic Records,Sheet:Top,,Core:C6
198,,,Mathematics,Sheet:Top,,Core:C7
199,,,Punctuation and Whitespace,Sheet:Top,,Core:C8
200
201
202
203
204
205
206
207
208
209
210
211
212
213
214
215
216
217
218
219
220
221
222
223
224
225
226
227
228
229
230
231
232
233
234
235
236
237
238
239
240
241
242
243
244
245
246
247
248
249
250
251
252
253
254
255
256
257
258
259
260
261
262
263
264
265
266
267
268
269
270
271
272
273
274
275
276
277
278
279
280
281
282
283
284
285
286
287
288
289
290
291
292
293
294
295
296
297
298
299
300
301
302
303
304
305
```

## Dc → DCE 3.0a

```csv
Dc ID,DCE 3.0a mapping
0,00
1,40
2
3
4
5
6
7
8
9
10
11
12
13,FD
14,FE
15,02
16,01
17,FF
18,20
19,21
20,22
21,23
22,24
23,25
24,26
25,27
26,28
27,29
28,2A
29,2B
30,2C
31,2D
32,2E
33,2F
34,30
35,31
36,32
37,33
38,34
39,35
40,36
41,37
42,38
43,39
44,3A
45,3B
46,3C
47,3D
48,3E
49,3F
50,41
51,42
52,43
53,44
54,45
55,46
56,47
57,48
58,49
59,4A
60,4B
61,4C
62,4D
63,4E
64,4F
65,50
66,51
67,52
68,53
69,54
70,55
71,56
72,57
73,58
74,59
75,5A
76,5B
77,5C
78,5D
79,5E
80,5F
81,60
82,61
83,62
84,63
85,64
86,65
87,66
88,67
89,68
90,69
91,6A
92,6B
93,6C
94,6D
95,6E
96,6F
97,70
98,71
99,72
100,73
101,74
102,75
103,76
104,77
105,78
106,79
107,7A
108,7B
109,7C
110,7D
111,7E
112
113,01
114,80
115,81
```

## Base64

```csv
Base64,Dc
A,127,Base 64 encapsulation digit 0
B,128,Base 64 encapsulation digit 1
C,129,Base 64 encapsulation digit 2
D,130,Base 64 encapsulation digit 3
E,131,Base 64 encapsulation digit 4
F,132,Base 64 encapsulation digit 5
G,133,Base 64 encapsulation digit 6
H,134,Base 64 encapsulation digit 7
I,135,Base 64 encapsulation digit 8
J,136,Base 64 encapsulation digit 9
K,137,Base 64 encapsulation digit 10
L,138,Base 64 encapsulation digit 11
M,139,Base 64 encapsulation digit 12
N,140,Base 64 encapsulation digit 13
O,141,Base 64 encapsulation digit 14
P,142,Base 64 encapsulation digit 15
Q,143,Base 64 encapsulation digit 16
R,144,Base 64 encapsulation digit 17
S,145,Base 64 encapsulation digit 18
T,146,Base 64 encapsulation digit 19
U,147,Base 64 encapsulation digit 20
V,148,Base 64 encapsulation digit 21
W,149,Base 64 encapsulation digit 22
X,150,Base 64 encapsulation digit 23
Y,151,Base 64 encapsulation digit 24
Z,152,Base 64 encapsulation digit 25
a,153,Base 64 encapsulation digit 26
b,154,Base 64 encapsulation digit 27
c,155,Base 64 encapsulation digit 28
d,156,Base 64 encapsulation digit 29
e,157,Base 64 encapsulation digit 30
f,158,Base 64 encapsulation digit 31
g,159,Base 64 encapsulation digit 32
h,160,Base 64 encapsulation digit 33
I,161,Base 64 encapsulation digit 34
j,162,Base 64 encapsulation digit 35
k,163,Base 64 encapsulation digit 36
l,164,Base 64 encapsulation digit 37
m,165,Base 64 encapsulation digit 38
n,166,Base 64 encapsulation digit 39
o,167,Base 64 encapsulation digit 40
p,168,Base 64 encapsulation digit 41
q,169,Base 64 encapsulation digit 42
r,170,Base 64 encapsulation digit 43
s,171,Base 64 encapsulation digit 44
t,172,Base 64 encapsulation digit 45
u,173,Base 64 encapsulation digit 46
v,174,Base 64 encapsulation digit 47
w,175,Base 64 encapsulation digit 48
x,176,Base 64 encapsulation digit 49
y,177,Base 64 encapsulation digit 50
z,178,Base 64 encapsulation digit 51
0,179,Base 64 encapsulation digit 52
1,180,Base 64 encapsulation digit 53
2,181,Base 64 encapsulation digit 54
3,182,Base 64 encapsulation digit 55
4,183,Base 64 encapsulation digit 56
5,184,Base 64 encapsulation digit 57
6,185,Base 64 encapsulation digit 58
7,186,Base 64 encapsulation digit 59
8,187,Base 64 encapsulation digit 60
9,188,Base 64 encapsulation digit 61
+,189,Base 64 encapsulation digit 62
/,190,Base 64 encapsulation digit 63
=,195,Base 64 encapsulation padding character
```

## Unicode → Dc

```csv
Unicode,Dc
0,0
8,116
9,117
A,120
B,118
C,193
D,121
20,18
21,19
22,20
23,21
24,22
25,23
26,24
27,25
28,26
29,27
2A,28
2B,29
2C,30
2D,31
2E,32
2F,33
30,34
31,35
32,36
33,37
34,38
35,39
36,40
37,41
38,42
39,43
3A,44
3B,45
3C,46
3D,47
3E,48
3F,49
40,1
41,50
42,51
43,52
44,53
45,54
46,55
47,56
48,57
49,58
4A,59
4B,60
4C,61
4D,62
4E,63
4F,64
50,65
51,66
52,67
53,68
54,69
55,70
56,71
57,72
58,73
59,74
5A,75
5B,76
5C,77
5D,78
5E,79
5F,80
60,81
61,82
62,83
63,84
64,85
65,86
66,87
67,88
68,89
69,90
6A,91
6B,92
6C,93
6D,94
6E,95
6F,96
70,97
71,98
72,99
73,100
74,101
75,102
76,103
77,104
78,105
79,106
7A,107
7B,108
7C,109
7D,110
7E,111
2010,31
```

## CDCE

```csv
Dc,HTML,Legacy CDCE data
1,@
2,"<font color=""red"">(begin separator)</font>"
3,"<font color=""red"">(end separator)</font>"
4,"<font color=""red""> to </font>"
5,"<font color=""red""> and </font>"
6,"<font color=""red"">(begin number)</font>"
7,"<font color=""red"">(end number)</font>"
8,"<font color=""red"">#</font>"
9,"<font color=""red""> Gregorian calendar </font>"
10,"<font color=""red"">+</font>"
11,"<font color=""red"">-</font>"
12,.
```

## Blank map

```csv
Map position,Dc ID,Dc Name,Flags
00,0,Null,Immutable
01
02
03
04
05
06
07
08
09
0A
0B
0C
0D
0E
0F
10
11
12
13
14
15
16
17
18
19
1A
1B
1C
1D
1E
1F
20
21
22
23
24
25
26
27
28
29
2A
2B
2C
2D
2E
2F
30
31
32
33
34
35
36
37
38
39
3A
3B
3C
3D
3E
3F
40
41
42
43
44
45
46
47
48
49
4A
4B
4C
4D
4E
4F
50
51
52
53
54
55
56
57
58
59
5A
5B
5C
5D
5E
5F
60
61
62
63
64
65
66
67
68
69
6A
6B
6C
6D
6E
6F
70
71
72
73
74
75
76
77
78
79
7A
7B
7C
7D
7E
7F
80
81
82
83
84
85
86
87
88
89
8A
8B
8C
8D
8E
8F
90
91
92
93
94
95
96
97
98
99
9A
9B
9C
9D
9E
9F
A0
A1
A2
A3
A4
A5
A6
A7
A8
A9
AA
AB
AC
AD
AE
AF
B0
B1
B2
B3
B4
B5
B6
B7
B8
B9
BA
BB
BC
BD
BE
BF
C0
C1
C2
C3
C4
C5
C6
C7
C8
C9
CA
CB
CC
CD
CE
CF
D0
D1
D2
D3
D4
D5
D6
D7
D8
D9
DA
DB
DC
DD
DE
DF
E0
E1
E2
E3
E4
E5
E6
E7
E8
E9
EA
EB
EC
ED
EE
EF
F0
F1
F2
F3
F4
F5
F6
F7
F8
F9
FA
FB
FC
FD,13,Full drop back,Immutable
FE,14,Drop back,Immutable
FF,17,No mapping,Immutable
```

## Core

```csv
Map position,Dc ID,Dc Name,Flags,This sheet contains important common entities
00,0,Null,Immutable
01,16,DCE Version Selectors,Immutable
02,15,Begin DCE content,Immutable
03,113,End DCE content,Immutable
04
05
06
07
08,116,Backspace
09,117,Horizontal tabulation
0A,120,Line feed
0B,118,Vertical tabulation
0C,193,Form feed
0D,121,ASCII carriage return
0E
0F
10
11
12
13
14
15
16
17
18
19
1A
1B
1C
1D
1E
1F
20,18,Word separator
21,19,Exclamation
22,20,Double quotation mark
23,21,Number sign
24,22,Dollar
25,23,Percent
26,24,And
27,25,Apostrophe
28,26,Left parenthesis
29,27,Right parenthesis
2A,28,Asterisk
2B,29,Plus
2C,30,Comma
2D,31,Hyphen
2E,32,Full stop
2F,33,Solidus
30,34,Zero
31,35,One
32,36,Two
33,37,Three
34,38,Four
35,39,Five
36,40,Six
37,41,Seven
38,42,Eight
39,43,Nine
3A,44,Colon
3B,45,Semicolon
3C,46,Less than
3D,47,Equals
3E,48,Greater than
3F,49,Question
40,1,At
41,50,Latin Capital Letter A
42,51,Latin Capital Letter B
43,52,Latin Capital Letter C
44,53,Latin Capital Letter D
45,54,Latin Capital Letter E
46,55,Latin Capital Letter F
47,56,Latin Capital Letter G
48,57,Latin Capital Letter H
49,58,Latin Capital Letter I
4A,59,Latin Capital Letter J
4B,60,Latin Capital Letter K
4C,61,Latin Capital Letter L
4D,62,Latin Capital Letter M
4E,63,Latin Capital Letter N
4F,64,Latin Capital Letter O
50,65,Latin Capital Letter P
51,66,Latin Capital Letter Q
52,67,Latin Capital Letter R
53,68,Latin Capital Letter S
54,69,Latin Capital Letter T
55,70,Latin Capital Letter U
56,71,Latin Capital Letter V
57,72,Latin Capital Letter W
58,73,Latin Capital Letter X
59,74,Latin Capital Letter Y
5A,75,Latin Capital Letter Z
5B,76,Left bracket
5C,77,Reversed solidus
5D,78,Right bracket
5E,79,Circumflex punctuation
5F,80,Underscore
60,81,Grave punctuation
61,82,Latin Small Letter A
62,83,Latin Small Letter B
63,84,Latin Small Letter C
64,85,Latin Small Letter D
65,86,Latin Small Letter E
66,87,Latin Small Letter F
67,88,Latin Small Letter G
68,89,Latin Small Letter H
69,90,Latin Small Letter I
6A,91,Latin Small Letter J
6B,92,Latin Small Letter K
6C,93,Latin Small Letter L
6D,94,Latin Small Letter M
6E,95,Latin Small Letter N
6F,96,Latin Small Letter O
70,97,Latin Small Letter P
71,98,Latin Small Letter Q
72,99,Latin Small Letter R
73,100,Latin Small Letter S
74,101,Latin Small Letter T
75,102,Latin Small Letter U
76,103,Latin Small Letter V
77,104,Latin Small Letter W
78,105,Latin Small Letter X
79,106,Latin Small Letter Y
7A,107,Latin Small Letter Z
7B,108,Left brace
7C,109,Vertical line
7D,110,Right brace
7E,111,Tilde punctuation
7F
80,114,Begin unbounded content
81,115,End unbounded content
82,127,Base 64 encapsulation digit 0
83,128,Base 64 encapsulation digit 1
84,129,Base 64 encapsulation digit 2
85,130,Base 64 encapsulation digit 3
86,131,Base 64 encapsulation digit 4
87,132,Base 64 encapsulation digit 5
88,133,Base 64 encapsulation digit 6
89,134,Base 64 encapsulation digit 7
8A,135,Base 64 encapsulation digit 8
8B,136,Base 64 encapsulation digit 9
8C,137,Base 64 encapsulation digit 10
8D,138,Base 64 encapsulation digit 11
8E,139,Base 64 encapsulation digit 12
8F,140,Base 64 encapsulation digit 13
90,141,Base 64 encapsulation digit 14
91,142,Base 64 encapsulation digit 15
92,143,Base 64 encapsulation digit 16
93,144,Base 64 encapsulation digit 17
94,145,Base 64 encapsulation digit 18
95,146,Base 64 encapsulation digit 19
96,147,Base 64 encapsulation digit 20
97,148,Base 64 encapsulation digit 21
98,149,Base 64 encapsulation digit 22
99,150,Base 64 encapsulation digit 23
9A,151,Base 64 encapsulation digit 24
9B,152,Base 64 encapsulation digit 25
9C,153,Base 64 encapsulation digit 26
9D,154,Base 64 encapsulation digit 27
9E,155,Base 64 encapsulation digit 28
9F,156,Base 64 encapsulation digit 29
A0,157,Base 64 encapsulation digit 30
A1,158,Base 64 encapsulation digit 31
A2,159,Base 64 encapsulation digit 32
A3,160,Base 64 encapsulation digit 33
A4,161,Base 64 encapsulation digit 34
A5,162,Base 64 encapsulation digit 35
A6,163,Base 64 encapsulation digit 36
A7,164,Base 64 encapsulation digit 37
A8,165,Base 64 encapsulation digit 38
A9,166,Base 64 encapsulation digit 39
AA,167,Base 64 encapsulation digit 40
AB,168,Base 64 encapsulation digit 41
AC,169,Base 64 encapsulation digit 42
AD,170,Base 64 encapsulation digit 43
AE,171,Base 64 encapsulation digit 44
AF,172,Base 64 encapsulation digit 45
B0,173,Base 64 encapsulation digit 46
B1,174,Base 64 encapsulation digit 47
B2,175,Base 64 encapsulation digit 48
B3,176,Base 64 encapsulation digit 49
B4,177,Base 64 encapsulation digit 50
B5,178,Base 64 encapsulation digit 51
B6,179,Base 64 encapsulation digit 52
B7,180,Base 64 encapsulation digit 53
B8,181,Base 64 encapsulation digit 54
B9,182,Base 64 encapsulation digit 55
BA,183,Base 64 encapsulation digit 56
BB,184,Base 64 encapsulation digit 57
BC,185,Base 64 encapsulation digit 58
BD,186,Base 64 encapsulation digit 59
BE,187,Base 64 encapsulation digit 60
BF,188,Base 64 encapsulation digit 61
C0,189,Base 64 encapsulation digit 62
C1,190,Base 64 encapsulation digit 63
C2,195,Base 64 encapsulation padding character
C3,191,Begin encapsulated UTF-8
C4,192,End encapsulated UTF-8
C5,>Variant_Selectors,Variant Selectors
C6,>Semantic_Records,Semantic Records
C7,>Mathematics,Mathematics
C8,>Punctuation_and_Whitespace,Punctuation and Whitespace
C9
CA
CB
CC
CD
CE
CF
D0
D1
D2
D3
D4
D5
D6
D7
D8
D9
DA
DB
DC
DD
DE
DF
E0
E1
E2
E3
E4
E5
E6
E7
E8
E9
EA
EB
EC
ED
EE
EF
F0
F1
F2
F3
F4
F5
F6
F7
F8
F9
FA
FB
FC
FD,13,Full drop back,Immutable
FE,14,Drop back,Immutable
FF,17,No mapping,Immutable
```

## DCE Version Selectors

```csv
Map position,Dc ID,Dc Name,Flags,Assigned characters in this map are immutable.
00,0,Null,Immutable
01,112,DCE Version Selector 3.0a,Immutable
02,194,DCE Version Selector 3.01a,Immutable
03
04
05
06
07
08
09
0A
0B
0C
0D
0E
0F
10
11
12
13
14
15
16
17
18
19
1A
1B
1C
1D
1E
1F
20
21
22
23
24
25
26
27
28
29
2A
2B
2C
2D
2E
2F
30
31
32
33
34
35
36
37
38
39
3A
3B
3C
3D
3E
3F
40
41
42
43
44
45
46
47
48
49
4A
4B
4C
4D
4E
4F
50
51
52
53
54
55
56
57
58
59
5A
5B
5C
5D
5E
5F
60
61
62
63
64
65
66
67
68
69
6A
6B
6C
6D
6E
6F
70
71
72
73
74
75
76
77
78
79
7A
7B
7C
7D
7E
7F
80
81
82
83
84
85
86
87
88
89
8A
8B
8C
8D
8E
8F
90
91
92
93
94
95
96
97
98
99
9A
9B
9C
9D
9E
9F
A0
A1
A2
A3
A4
A5
A6
A7
A8
A9
AA
AB
AC
AD
AE
AF
B0
B1
B2
B3
B4
B5
B6
B7
B8
B9
BA
BB
BC
BD
BE
BF
C0
C1
C2
C3
C4
C5
C6
C7
C8
C9
CA
CB
CC
CD
CE
CF
D0
D1
D2
D3
D4
D5
D6
D7
D8
D9
DA
DB
DC
DD
DE
DF
E0
E1
E2
E3
E4
E5
E6
E7
E8
E9
EA
EB
EC
ED
EE
EF
F0
F1
F2
F3
F4
F5
F6
F7
F8
F9
FA
FB
FC
FD,13,Full drop back,Immutable
FE,14,Drop back,Immutable
FF,17,No mapping,Immutable
```

## Variant Selectors

```csv
Map position,Dc ID,Dc Name,Flags
00,0,Null,Immutable
01,122,Variant Selector 1
02,123,Variant Selector 2
03,124,Variant Selector 3
04,125,Variant Selector 4
05,126,Variant Selector 5
06
07
08
09
0A
0B
0C
0D
0E
0F
10
11
12
13
14
15
16
17
18
19
1A
1B
1C
1D
1E
1F
20
21
22
23
24
25
26
27
28
29
2A
2B
2C
2D
2E
2F
30
31
32
33
34
35
36
37
38
39
3A
3B
3C
3D
3E
3F
40
41
42
43
44
45
46
47
48
49
4A
4B
4C
4D
4E
4F
50
51
52
53
54
55
56
57
58
59
5A
5B
5C
5D
5E
5F
60
61
62
63
64
65
66
67
68
69
6A
6B
6C
6D
6E
6F
70
71
72
73
74
75
76
77
78
79
7A
7B
7C
7D
7E
7F
80
81
82
83
84
85
86
87
88
89
8A
8B
8C
8D
8E
8F
90
91
92
93
94
95
96
97
98
99
9A
9B
9C
9D
9E
9F
A0
A1
A2
A3
A4
A5
A6
A7
A8
A9
AA
AB
AC
AD
AE
AF
B0
B1
B2
B3
B4
B5
B6
B7
B8
B9
BA
BB
BC
BD
BE
BF
C0
C1
C2
C3
C4
C5
C6
C7
C8
C9
CA
CB
CC
CD
CE
CF
D0
D1
D2
D3
D4
D5
D6
D7
D8
D9
DA
DB
DC
DD
DE
DF
E0
E1
E2
E3
E4
E5
E6
E7
E8
E9
EA
EB
EC
ED
EE
EF
F0
F1
F2
F3
F4
F5
F6
F7
F8
F9
FA
FB
FC
FD,13,Full drop back,Immutable
FE,14,Drop back,Immutable
FF,17,No mapping,Immutable
```

## Semantic Records

```csv
Map position,Dc ID,Dc Name,Flags
00,0,Null,Immutable
01,2,Begin separator
02,3,End separator
03,4,Range
04,5,Record summation
05,6,Begin number
06,7,End number
07,8,Unknown value
08,9,Gregorian calendar identifier
09,10,Positive
0A,11,Negative
0B
0C
0D
0E
0F
10
11
12
13
14
15
16
17
18
19
1A
1B
1C
1D
1E
1F
20
21
22
23
24
25
26
27
28
29
2A
2B
2C
2D
2E
2F
30
31
32
33
34
35
36
37
38
39
3A
3B
3C
3D
3E
3F
40
41
42
43
44
45
46
47
48
49
4A
4B
4C
4D
4E
4F
50
51
52
53
54
55
56
57
58
59
5A
5B
5C
5D
5E
5F
60
61
62
63
64
65
66
67
68
69
6A
6B
6C
6D
6E
6F
70
71
72
73
74
75
76
77
78
79
7A
7B
7C
7D
7E
7F
80
81
82
83
84
85
86
87
88
89
8A
8B
8C
8D
8E
8F
90
91
92
93
94
95
96
97
98
99
9A
9B
9C
9D
9E
9F
A0
A1
A2
A3
A4
A5
A6
A7
A8
A9
AA
AB
AC
AD
AE
AF
B0
B1
B2
B3
B4
B5
B6
B7
B8
B9
BA
BB
BC
BD
BE
BF
C0
C1
C2
C3
C4
C5
C6
C7
C8
C9
CA
CB
CC
CD
CE
CF
D0
D1
D2
D3
D4
D5
D6
D7
D8
D9
DA
DB
DC
DD
DE
DF
E0
E1
E2
E3
E4
E5
E6
E7
E8
E9
EA
EB
EC
ED
EE
EF
F0
F1
F2
F3
F4
F5
F6
F7
F8
F9
FA
FB
FC
FD,13,Full drop back,Immutable
FE,14,Drop back,Immutable
FF,17,No mapping,Immutable
```

## Mathematics

```csv
Map position,Dc ID,Dc Name,Flags
00,0,Null,Immutable
01,12,Decimal point
02
03
04
05
06
07
08
09
0A
0B
0C
0D
0E
0F
10
11
12
13
14
15
16
17
18
19
1A
1B
1C
1D
1E
1F
20
21
22
23
24
25
26
27
28
29
2A
2B
2C
2D
2E
2F
30
31
32
33
34
35
36
37
38
39
3A
3B
3C
3D
3E
3F
40
41
42
43
44
45
46
47
48
49
4A
4B
4C
4D
4E
4F
50
51
52
53
54
55
56
57
58
59
5A
5B
5C
5D
5E
5F
60
61
62
63
64
65
66
67
68
69
6A
6B
6C
6D
6E
6F
70
71
72
73
74
75
76
77
78
79
7A
7B
7C
7D
7E
7F
80
81
82
83
84
85
86
87
88
89
8A
8B
8C
8D
8E
8F
90
91
92
93
94
95
96
97
98
99
9A
9B
9C
9D
9E
9F
A0
A1
A2
A3
A4
A5
A6
A7
A8
A9
AA
AB
AC
AD
AE
AF
B0
B1
B2
B3
B4
B5
B6
B7
B8
B9
BA
BB
BC
BD
BE
BF
C0
C1
C2
C3
C4
C5
C6
C7
C8
C9
CA
CB
CC
CD
CE
CF
D0
D1
D2
D3
D4
D5
D6
D7
D8
D9
DA
DB
DC
DD
DE
DF
E0
E1
E2
E3
E4
E5
E6
E7
E8
E9
EA
EB
EC
ED
EE
EF
F0
F1
F2
F3
F4
F5
F6
F7
F8
F9
FA
FB
FC
FD,13,Full drop back,Immutable
FE,14,Drop back,Immutable
FF,17,No mapping,Immutable
```

## Punctuation and Whitespace

```csv
Map position,Dc ID,Dc Name,Flags
00,0,Null,Immutable
01,119,New line
02
03
04
05
06
07
08
09
0A
0B
0C
0D
0E
0F
10
11
12
13
14
15
16
17
18
19
1A
1B
1C
1D
1E
1F
20
21
22
23
24
25
26
27
28
29
2A
2B
2C
2D
2E
2F
30
31
32
33
34
35
36
37
38
39
3A
3B
3C
3D
3E
3F
40
41
42
43
44
45
46
47
48
49
4A
4B
4C
4D
4E
4F
50
51
52
53
54
55
56
57
58
59
5A
5B
5C
5D
5E
5F
60
61
62
63
64
65
66
67
68
69
6A
6B
6C
6D
6E
6F
70
71
72
73
74
75
76
77
78
79
7A
7B
7C
7D
7E
7F
80
81
82
83
84
85
86
87
88
89
8A
8B
8C
8D
8E
8F
90
91
92
93
94
95
96
97
98
99
9A
9B
9C
9D
9E
9F
A0
A1
A2
A3
A4
A5
A6
A7
A8
A9
AA
AB
AC
AD
AE
AF
B0
B1
B2
B3
B4
B5
B6
B7
B8
B9
BA
BB
BC
BD
BE
BF
C0
C1
C2
C3
C4
C5
C6
C7
C8
C9
CA
CB
CC
CD
CE
CF
D0
D1
D2
D3
D4
D5
D6
D7
D8
D9
DA
DB
DC
DD
DE
DF
E0
E1
E2
E3
E4
E5
E6
E7
E8
E9
EA
EB
EC
ED
EE
EF
F0
F1
F2
F3
F4
F5
F6
F7
F8
F9
FA
FB
FC
FD,13,Full drop back,Immutable
FE,14,Drop back,Immutable
FF,17,No mapping,Immutable
```

## Dc to Base64

```csv
Dc,Base64
127,A
128,B
129,C
130,D
131,E
132,F
133,G
134,H
135,I
136,J
137,K
138,L
139,M
140,N
141,O
142,P
143,Q
144,R
145,S
146,T
147,U
148,V
149,W
150,X
151,Y
152,Z
153,a
154,b
155,c
156,d
157,e
158,f
159,g
160,h
161,I
162,j
163,k
164,l
165,m
166,n
167,o
168,p
169,q
170,r
171,s
172,t
173,u
174,v
175,w
176,x
177,y
178,z
179,0
180,1
181,2
182,3
183,4
184,5
185,6
186,7
187,8
188,9
189,+
190,/
195,=
```

## Translators and formats

```csv
Format code,Name,Translator comments,One-step,One-step quality,Read,Read quality,Write,Write quality,Description,Example notes,Example
CDCE
CDCE,cdce
CDCE (legacy),cdce_legacy,,,,Y,Lossy,,,CDCE format used by Weave.,,oauo@9@981g3ud @1@@2@1@3@@10@2012@2@2@3@10@2@2@3@7@2@2@3@15@2@2@3@@8@@2@2@3@@8@@2@2@3@@8@@4@@1@@2@1@3@@10@2012@2@2@3@11@2@2@3@@19@2@2@3@16@2@2@3@7@2@2@3@6@2@2@3@27@5@@1@@2@1@3@@10@2013@2@2@3@10@2@2@3@7@2@2@3@15@2@2@3@@8@@2@2@3@@8@@2@2@3@@8@ 
"CDCE (legacy, strict)",cdce_lstrict,,,,Y,Lossy
Generic DCE
DCE,dce,Currently defaults to 3.0a due to unreliability of the 3.01a translators.,,,Y,Lossless,Y,Release
DCE (hex-encoded),hex_dce,Relies on dce translators.,,,Y,,Y
DCE (raw hex-encoded),hex_dce_raw
DCE 3.0a
DCE 3.0a,3_0a,,,,(Y),Lossless,Y,Release,,Example encoded in hexadecimal.,44434565020101FD8048656C6C6F20576F726C642181FD03
DCE 3.0a (raw),dce_3_0a_raw,,,,,,,,,Example encoded in hexadecimal.,8048656C6C6F20576F726C642181
DCE 3.0a (hex-encoded),hex_3_0a,Relies on 3_0a translators.,,,(Y),Lossless,Y,Release,,,44434565020101FD8048656C6C6F20576F726C642181FD03
DCE 3.0a (raw hex-encoded),hex_3_0a_raw,,,,,,,,,,8048656C6C6F20576F726C642181
DCE 3.0a (old translator),3_0a_old,Deprecated,Y
dce2txt,dce2txt,Deprecated,Y
dce2hex,dce2hex,Deprecated,Y
hex2dce,hex2dce,Deprecated,Y
DCE 3.01a
DCE 3.01a,3_01a,,,,(Y),Lossless,Y,Very poor,,Example encoded in hexadecimal.,44434565020102FD8048656C6C6F20576F726C642181FD03
DCE 3.01a (raw),dce_3_01a_raw,,,,,,,,,Example encoded in hexadecimal.,8048656C6C6F20576F726C642181
DCE 3.01a (hex-encoded),hex_3_01a,Relies on dce translators.,,,Y,,Y,,,,44434565020102FD8048656C6C6F20576F726C642181FD03
DCE 3.01a (raw hex-encoded),hex_3_01a_raw,,,,,,,,,,8048656C6C6F20576F726C642181
Miscellaneous DCE
Dc ID list (ASCII),dc,Lossless output for Dc and DCE 3.0a input.,,,Y,Lossless,Y,Depends on input format,Used internally by libdce.,,"96,82,102,96,9,43,42,35,88,37,102,85,18,1,2,35,3,10,36,34,35,36,2,36,3,35,34,2,36,3,41,2,36,3,35,39,2,36,3,8,2,36,3,8,2,36,3,8,4,1,2,35,3,10,36,34,35,36,2,36,3,35,35,2,36,3,1,35,43,2,36,3,35,40,2,36,3,41,2,36,3,40,2,36,3,36,41,5,1,2,35,3,10,36,34,35,37,2,36,3,35,34,2,36,3,41,2,36,3,35,39,2,36,3,8,2,36,3,8,2,36,3,8,18"
Unicode
UTF-8,utf8,Write translator may need adaptation for properly detecting encapsulated UTF-8 content,,,Y,Lossy,Y,Release
UTF-32,utf32,Relies on utf8 translators.,,,Y,Lossy,Y,Release
UTF-8 (Base64-encoded),utf8_base64,Relies on utf8 translators.,,,Y,,Y
"UTF-8 (Base64-encoded, ASCII Dc ID list)",utf8_dc64,Uses utf8_base64 translators.,,,Y,Very poor,Y,Very poor,,,"145,133,148,171,154,133,187,159,148,181,188,177,154,133,143,160"
"UTF-8 (Base64-encoded, DCE 3.01a+ encoding)",utf8_dc64_bin,Uses utf8_dc64_bin translators.,,,,,Y,Very poor,,Example encoded in hexadecimal.,948897AE9D88BEA297B8BFB49D8892A3
HTML
HTML,html
HTML (snippet),html_snippet
HTML (legacy CDCE output),html_l,Relies on html_snippet_l translator.,,,,,Y,Very poor,,,"<html><head><title></title></head><body>oauo<font color=""red""> Gregorian calendar </font>981g3ud @<font color=""red"">(begin separator)</font>1<font color=""red"">(end separator)</font><font color=""red"">+</font>2012<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>10<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>7<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>15<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red""> to </font>@<font color=""red"">(begin separator)</font>1<font color=""red"">(end separator)</font><font color=""red"">+</font>2012<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>11<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>@19<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>16<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>7<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>6<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>27<font color=""red""> and </font>@<font color=""red"">(begin separator)</font>1<font color=""red"">(end separator)</font><font color=""red"">+</font>2013<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>10<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>7<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>15<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font> </body></html>"
HTML (snippet) (legacy CDCE output),html_snippet_l,,,,,,Y,Very poor,,,"oauo<font color=""red""> Gregorian calendar </font>981g3ud @<font color=""red"">(begin separator)</font>1<font color=""red"">(end separator)</font><font color=""red"">+</font>2012<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>10<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>7<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>15<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red""> to </font>@<font color=""red"">(begin separator)</font>1<font color=""red"">(end separator)</font><font color=""red"">+</font>2012<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>11<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>@19<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>16<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>7<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>6<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>27<font color=""red""> and </font>@<font color=""red"">(begin separator)</font>1<font color=""red"">(end separator)</font><font color=""red"">+</font>2013<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>10<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>7<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font>15<font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font><font color=""red"">(begin separator)</font>2<font color=""red"">(end separator)</font><font color=""red"">#</font> "
```
