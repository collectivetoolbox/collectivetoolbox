# DCE, version 3.0a r2 (26 December 2012)

## DCE 3.0a

```csv
"DCE, version 3.0a r2 (26 December 2012)
Note that this is a minor revision of DCE 3.0a, and does not change any entity mappings.
"

The system specified herein is an information representation
"system and binary file format. Most DCE entities are represented as single 8-bit bytes, and the mappings of byte to"
entity are determined by switching entities for an effectively unlimited set of entities. Some mapping sheets may use more
bytes per entity.

The required structure of a file is as follows:

First four bytes should be: 44434565 (ASCII DCEe)
Fifth byte should be: 02 (Begin DCE content)
Sixth byte should be: 01 (DCE version selector byte)
"Seventh to n bytes should be: DCE version mapping bytes, until a byte FD is reached, which indicates a complete"
drop back to the core mapping sheet.
Next bytes should be: Document content
Next to last byte should be: FD (full drop back)
Final byte should be: 03 (end DCE content)
Example document (Hello World!): 44 43 45 65 02 01 01 FD 80 48 65 6C 6C 6F 20 57 6F 72 6C 64 21 81 FD 03


Notes about this specification:

"1) Mapping sheets always map byte 00 to DCE Character (Dc) 0, “Null”. This is an immutable mapping."
"2) Mapping sheets always map byte FD to Dc 13, “Full drop back”. This is an immutable mapping."
"3) Mapping sheets always map byte FE to Dc 14, “Drop back”. This is an immutable mapping."
"4) Mapping sheets always leave byte FF unmapped, for processing use. This is an immutable mapping."
5) The printable ASCII characters and the ASCII space are mapped in Core at their ASCII byte values for compatibility.
6) Round trip mappings with Unicode or ASCII are not yet supported by this standard.

Definitions:

Boundings: Document structure information to provide a context for one or more DCE strings
Immutable: A flag on a mapping that indicates that this assignment will not be changed in future versions
of the standard. This is needed for characters necessary to the correct functioning of the
standard with other revisions.
Unbounded: Indicates that a given DCE string has no boundings.
```

## DCE Character List by ID

```csv
Dc ID or ID Sequence,Dc Decomposition,Dc Name,Dc Class,Character or representation,Mappings,Description and specification,Annotations,Other names,See also,Internal ASCII representation (deprecated),Unicode mapping,Mojikyo mapping,TRON mapping
0,,Null,Technical:Controls,␀,[All sheets: 00],,,,,,00
1,,At,General:Symbols,@,Core:40,@. Needed in CDCE prior to main DCE specification for syntactical purposes.,,,,,40
2,,Begin separator,Semantic:Syntactical,(begin separator),,Begin a DCE separator string.
3,,End separator,Semantic:Syntactical,(end separator),,End a DCE separator string.
4,,Range,Semantic:Syntactical,to ,,The DCE range character. Used to represent ranges in DCE. See the wiki for more information.,,,,\>
5,,Record summation,Semantic:Syntactical,and ,,Used to denote the union of two records. See the wiki for more information.,,,,=
6,,Begin number,Semantic:Syntactical,(begin number),,Begin a DCE number.
7,,End number,Semantic:Syntactical,(end number),,End a DCE number.
8,,Unknown value,Semantic:Substitution,#,,Substitution character for an unknown value.,,,,#
9,,Gregorian calendar identifier,Semantic:Syntactical,Gregorian calendar ,,Internal DCE calendar identifier for the Gregorian calendar.,,,,G
10,,Positive,Semantic:Syntactical,+,,Positive value (syntactical representation — This is not the + character),,,,+
11,,Negative,Semantic:Syntactical,-,,Negative (syntactical representation — This is not the − character),,,,-
12,,Decimal point,General:Mathematics,",",,,,,,.,2E
13,,Full drop back,Technical:Internal,(full drop back),[All sheets: FD],Drop back to the core sheet
14,,Drop back,Technical:Internal,(drop back),[All sheets: FE],Drop back to the enclosing sheet
15,,Begin DCE content,Technical:Internal,(begin file),Core:02,Begins a DCE region
16,,DCE Version Selectors,Sheet:Top,,Core:01
17,,No mapping,Technical:Internal,,[All sheets: FF],Processing use only (not for interchange)
18,,Word separator,General:Punctuation,,Core:20,,,,,,20
19,,Exclamation,General:Punctuation,!,Core:21,,,,,,21
20,,Double quotation mark,General:Punctuation,"""",Core:22,,,,,,22
21,,Number sign,General:Number,#,Core:23,,,"octothorpe, hash, crosshatch",,,23
22,,Dollar,General:Financial,$,Core:24,,,,,,24
23,,Percent,General:Symbols,%,Core:25,,,,,,25
24,,And,General:Symbols,&,Core:26,,,,,,26
25,,Apostrophe,General:Punctuation,',Core:27,,,,,,27
26,,Left parenthesis,General:Punctuation,(,Core:28,,,left bracket,,,28
27,,Right parenthesis,General:Punctuation,),Core:29,,,right bracket,,,29
28,,Asterisk,General:Symbols,*,Core:2A,,,star,,,2A
29,,Plus,General:Mathematics,+,Core:2B,,,,,,2B
30,,Comma,General:Punctuation,",",Core:2C,,,,,,2C
31,,Hyphen,General:Punctuation,‐,Core:2D,,,,,,2010
32,,Full stop,General:Punctuation,.,Core:2E,,,period,,,2E
33,,Solidus,General:Punctuation,/,Core:2F,,,"virgule, stroke, slash",,,2F
34,,Zero,General:Mathematics,0,Core:30,,,,,,30
35,,One,General:Mathematics,1,Core:31,,,,,,31
36,,Two,General:Mathematics,2,Core:32,,,,,,32
37,,Three,General:Mathematics,3,Core:33,,,,,,33
38,,Four,General:Mathematics,4,Core:34,,,,,,34
39,,Five,General:Mathematics,5,Core:35,,,,,,35
40,,Six,General:Mathematics,6,Core:36,,,,,,36
41,,Seven,General:Mathematics,7,Core:37,,,,,,37
42,,Eight,General:Mathematics,8,Core:38,,,,,,38
43,,Nine,General:Mathematics,9,Core:39,,,,,,39
44,,Colon,General:Punctuation,:,Core:3A,,,,,,3A
45,,Semicolon,General:Punctuation,;,Core:3B,,,,,,3B
46,,Less than,General:Mathematics,<,Core:3C,,,,,,3C
47,,Equals,General:Mathematics,=,Core:3D,,,,,,3D
48,,Greater than,General:Mathematics,>,Core:3E,,,,,,3E
49,,Question,General:Punctuation,?,Core:3F,,,,,,3F
50,,Latin Capital Letter A,Script:Latin,A,Core:41,,,,,,41
51,,Latin Capital Letter B,Script:Latin,B,Core:42,,,,,,42
52,,Latin Capital Letter C,Script:Latin,C,Core:43,,,,,,43
53,,Latin Capital Letter D,Script:Latin,D,Core:44,,,,,,44
54,,Latin Capital Letter E,Script:Latin,E,Core:45,,,,,,45
55,,Latin Capital Letter F,Script:Latin,F,Core:46,,,,,,46
56,,Latin Capital Letter G,Script:Latin,G,Core:47,,,,,,47
57,,Latin Capital Letter H,Script:Latin,H,Core:48,,,,,,48
58,,Latin Capital Letter I,Script:Latin,I,Core:49,,,,,,49
59,,Latin Capital Letter J,Script:Latin,J,Core:4A,,,,,,4A
60,,Latin Capital Letter K,Script:Latin,K,Core:4B,,,,,,4B
61,,Latin Capital Letter L,Script:Latin,L,Core:4C,,,,,,4C
62,,Latin Capital Letter M,Script:Latin,M,Core:4D,,,,,,4D
63,,Latin Capital Letter N,Script:Latin,N,Core:4E,,,,,,4E
64,,Latin Capital Letter O,Script:Latin,O,Core:4F,,,,,,4F
65,,Latin Capital Letter P,Script:Latin,P,Core:50,,,,,,50
66,,Latin Capital Letter Q,Script:Latin,Q,Core:51,,,,,,51
67,,Latin Capital Letter R,Script:Latin,R,Core:52,,,,,,52
68,,Latin Capital Letter S,Script:Latin,S,Core:53,,,,,,53
69,,Latin Capital Letter T,Script:Latin,T,Core:54,,,,,,54
70,,Latin Capital Letter U,Script:Latin,U,Core:55,,,,,,55
71,,Latin Capital Letter V,Script:Latin,V,Core:56,,,,,,56
72,,Latin Capital Letter W,Script:Latin,W,Core:57,,,,,,57
73,,Latin Capital Letter X,Script:Latin,X,Core:58,,,,,,58
74,,Latin Capital Letter Y,Script:Latin,Y,Core:59,,,,,,59
75,,Latin Capital Letter Z,Script:Latin,Z,Core:5A,,,,,,5A
76,,Left bracket,General:Punctuation,[,Core:5B,,,left square bracket,,,5B
77,,Reversed solidus,General:Punctuation,\,Core:5C,,,backslash,,,5C
78,,Right bracket,General:Punctuation,],Core:5D,,,right square bracket,,,5D
79,,Circumflex punctuation,General:Punctuation,^,Core:5E,,,,,,5E
80,,Underscore,General:Punctuation,_,Core:5F,,,low line,,,5F
81,,Grave punctuation,General:Punctuation,`,Core:60,,,,,,60
82,,Latin Small Letter A,Script:Latin,a,Core:61,,,,,,61
83,,Latin Small Letter B,Script:Latin,b,Core:62,,,,,,62
84,,Latin Small Letter C,Script:Latin,c,Core:63,,,,,,63
85,,Latin Small Letter D,Script:Latin,d,Core:64,,,,,,64
86,,Latin Small Letter E,Script:Latin,e,Core:65,,,,,,65
87,,Latin Small Letter F,Script:Latin,f,Core:66,,,,,,66
88,,Latin Small Letter G,Script:Latin,g,Core:67,,,,,,67
89,,Latin Small Letter H,Script:Latin,h,Core:68,,,,,,68
90,,Latin Small Letter I,Script:Latin,I,Core:69,,,,,,69
91,,Latin Small Letter J,Script:Latin,j,Core:6A,,,,,,6A
92,,Latin Small Letter K,Script:Latin,k,Core:6B,,,,,,6B
93,,Latin Small Letter L,Script:Latin,l,Core:6C,,,,,,6C
94,,Latin Small Letter M,Script:Latin,m,Core:6D,,,,,,6D
95,,Latin Small Letter N,Script:Latin,n,Core:6E,,,,,,6E
96,,Latin Small Letter O,Script:Latin,o,Core:6F,,,,,,6F
97,,Latin Small Letter P,Script:Latin,p,Core:70,,,,,,70
98,,Latin Small Letter Q,Script:Latin,q,Core:71,,,,,,71
99,,Latin Small Letter R,Script:Latin,r,Core:72,,,,,,72
100,,Latin Small Letter S,Script:Latin,s,Core:73,,,,,,73
101,,Latin Small Letter T,Script:Latin,t,Core:74,,,,,,74
102,,Latin Small Letter U,Script:Latin,u,Core:75,,,,,,75
103,,Latin Small Letter V,Script:Latin,v,Core:76,,,,,,76
104,,Latin Small Letter W,Script:Latin,w,Core:77,,,,,,77
105,,Latin Small Letter X,Script:Latin,x,Core:78,,,,,,78
106,,Latin Small Letter Y,Script:Latin,y,Core:79,,,,,,79
107,,Latin Small Letter Z,Script:Latin,z,Core:7A,,,,,,7A
108,,Left brace,General:Punctuation,{,Core:7B,,,left curly bracket,,,7B
109,,Vertical line,General:Punctuation,|,Core:7C,,,"vertical bar, pipe",,,7C
110,,Right brace,General:Punctuation,},Core:7D,,,right curly bracket,,,7D
111,,Tilde punctuation,General:Punctuation,~,Core:7E,,,,,,7E
112,,DCE Version Selector 3.0a,Technical:Version,,DCE Version Selectors:01
113,,End DCE content,Technical:Internal,(end file),Core:01,Ends a DCE region
114,,Begin unbounded content,Semantic:Structure,,Core:80,Begins an unbounded DCE string
115,,End unbounded content,Semantic:Structure,,Core:81,Ends an unbounded DCE string
116
117
118
119
120
121
122
123
124
125
126
127
128
129
130
131
132
133
134
135
136
137
138
139
140
141
142
143
144
145
146
147
148
149
150
151
152
153
154
155
156
157
158
159
160
161
162
163
164
165
166
167
168
169
170
171
172
173
174
175
176
177
178
179
180
181
182
183
184
185
186
187
188
189
190
191
192
193
194
195
196
197
198
199
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

## Dc → Unicode

```csv
Dc ID or ID Sequence,Unicode mapping
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
12,2E
13
14
15
16
17
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
31,2010
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
113
114
115
116
117
118
119
120
121
122
123
124
125
126
127
128
129
130
131
132
133
134
135
136
137
138
139
140
141
142
143
144
145
146
147
148
149
150
151
152
153
154
155
156
157
158
159
160
161
162
163
164
165
166
167
168
169
170
171
172
173
174
175
176
177
178
179
180
181
182
183
184
185
186
187
188
189
190
191
192
193
194
195
196
197
198
199
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

## Unicode → Dc

```csv
Unicode,Dc
00,0
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

## DCE Version Selectors

```csv
Map position,Dc ID,Dc Name,Flags,Assigned characters in this map are immutable.
00,0,Null,Immutable
01,112,DCE Version Selector 3.0a,Immutable
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

## CDCE

```csv
Dc,HTML,,Legacy CDCE data
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
