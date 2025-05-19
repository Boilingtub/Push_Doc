# Push_Doc
Scriptable Application to Automate google document updates \
for routine automatable changes 

![alt text](https://github.com/Boilingtub/Push_Doc/blob/main/PushDocLogo.png?raw=true)


## Build Dependencies
- rust Compiler
## Runtime Dependencies
- client_secrets file 
## Installation
- build from source using `cargo b --release` 
- Precompiled Binaries available under tab on github `*releases*`
## Configuration
Create new file (extention does not matter) in a known location \
pass in file as commandline argument when running program \
```
./push_doc <file_path> 
```
example configuration available in "examples folder" of source code

## Configuration Keys
example :
```
{
id=1kVGyd1WW_qqcjFqf56YkET2Y_77Bct-FCZP0qCXl0yo
client_secrets=$[client_secret_1014602435348-6317hvuq1n0stf3p9hqlqnplls8cb336.apps.googleusercontent.com.json]$
insertText=(oogabooga,END)
replaceAllText=(hello,idkhw)
}
```
### id
id of google document you wish to edit,
found in url when you open the document in web app for google docs
Example url: `https://docs.google.com/document/d/1kVGyd1WW_qqcjFqf56YkET2Y_77Bct-FCZP0qCXl0yo/edit`
```
id=<The id of the google document you wish to edit>
```

### client_secrets
client_secrets , this a per user client_secrets file that is used to autherticate a user \

**NOTICE :** the file/path is surrounded by `$[ ]$` this tells push_doc to look for a file \
when you surround any text with `$[ ]$` then it is treated as a file path and that file content \
is used as the parameter.
```
client_secrets=<usually path of client_secrets file>
```

### insertText
specifies that text should be placed into the document \
second parameter contains the index where said text sould be places \
`insertText=(parm1,param2)` \
param1 can be `plain text` or `$[file/path]$` \
param2 can be either `a number` , or `END` or `START` \
`a number` number indicating place in document \
`END` always puts text at end of google document \
`START` alwys puts text at begining of google document \
```
insertText=(<text to be inserted>,<index at which text should be inserted>)
```

### replaceAllText
`replaceAllText=(parm1.parm2)`
all occurances of the text in param1 will be replaced by the text in param2
```
replaceAllText=(<Text to be replaced>,<text to replace with>)
```

### deleteContentRange
`deleteContentRange=(parm1,parm2)` \
parm1 cannot be larger than param2 ! \
params can be any `number > 0` , or `END` or `START`
```
deleteContentRange=(<start index>,<end index>)
```
## Security 
The program only ever uses the localhost:8000 thus the included
self-signed ca-certificate is enough to simply authorize localhost.
## Behaviour
script gets organised and each Update is run sequentially
## Known Bugs
## Roadmap for the future
- remove all unwraps and '?'-operators from the program
