MODULE:path/to/module

b/zl s/in
    test works with zero length string
        in s:
        out 1
    test works with nonzero length string
        in s:a
        out 0
    eq 0 len s/in

s/getFormatImportSetting s/a s/b
    s:UNIMPLEMENTED

s/dcGetMappingToFormat s/a s/b
    s:UNIMPLEMENTED

s/getEnvCodeLanguage//    s:UNIMPLEMENTED

b/nne n/a n/b
    : number not equal to (this is a comment; any bytes allowed until nl)
    false

b/sne s/a s/b
    false

l/append l/a l/b
    : append one list to another
    : UNIMPLEMENTED
    test works
        in l:0 1 l:2 3
        out l:0 1 2 3
    l:0 3 2 1

s/getTargetLanguageForDctCodeToText
    new s/targetLanguage getFormatImportSetting s:codetotext s:language
    when zl s/targetLanguage : getEnvCodeLanguage : s/targetLanguage

l/dcaFromElad s/in
    : UNIMPLEMENTED
    l:0 1 2 3

l/push l/in n/toPush
    : UNIMPLEMENTED

l/convertElementAndAppend n/e s/targetLanguage l/converted
    new s/temp dcGetMappingToFormat n/e s/targetLanguage
    when ne 0 len s/temp : : lappend l/res dcaFromElad s/temp : : push l/res n/currentDc
    : Alternatively, can indent arguments for readability (this is still one logical line). Only one level of indentation allowed.
    when
        ne 0 len s/temp : :
        lappend l/res dcaFromElad s/temp : :
        push l/res n/currentDc

e:l/dctCodeToText l/in
    new l/res
    new s/targetLanguage getTargetLanguageForDctCodeToText
    each l/in set l/res convertElementAndAppend /e s/targetLanguage l/res
    l/res
