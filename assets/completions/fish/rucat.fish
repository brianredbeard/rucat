complete -c rucat -s f -l format -d 'Output format' -r -f -a "ansi\t'ANSI box drawing characters'
xml\t'XML format'
json\t'JSON format'
markdown\t'Markdown code blocks'
ascii\t'Simple ASCII header'
utf8\t'Fancy UTF-8 box drawing'
pretty\t'Pretty-printed with syntax highlighting'"
complete -c rucat -l ansi-width -d 'Width for ANSI formatting (excluding borders)' -r
complete -c rucat -l utf8-width -d 'Width for UTF8 formatting (excluding borders)' -r
complete -c rucat -l strip -d 'Remove N leading path components when printing filenames' -r
complete -c rucat -l pretty-syntax -d 'Explicitly set the syntax for the \'pretty\' formatter' -r
complete -c rucat -l clipboard-provider-for-test -d 'FOR TESTING ONLY: Force a specific clipboard provider' -r
complete -c rucat -s n -l numbers -d 'Add a gutter with line numbers'
complete -c rucat -s 0 -l null -d 'Read NUL-terminated file list from STDIN (like `xargs -0`)'
complete -c rucat -s c -l copy -d 'Copy output to the system clipboard'
complete -c rucat -s h -l help -d 'Print help (see more with \'--help\')'
complete -c rucat -s V -l version -d 'Print version'
