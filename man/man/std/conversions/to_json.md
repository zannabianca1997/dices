---
title: "The `to_json` intrisic"
---
# The `to_json` intrisic

The `to_json` intrisic convert a value into a JSON string.
```dices
#>>> let to_json = std.conversions.to_json;
>>> to_json(true)
"true"
>>> to_json(34)
"34"
>>> to_json([1,2,3])
"[1,2,3]"
>>> to_json(<|a:34, b:5|>)
# "{\"a\":34,\"b\":5}"
```
The original value can always be parsed back with [`from_json`](man:std/conversions/from_json).

## Simple scalars

Booleans, nulls and string are trivially converted into json:
```dices
#>>> let to_json = std.conversions.to_json;
>>> [to_json(true), to_json(false)]
["true","false"]
>>> to_json(null)
"null"
>>> to_json("Hello world")
"\"Hello world\""
```

Number are converted if they fit into 64 bits, if not they are expanded in a map with the key `$type` set to `number`:
```dices
#>>> let to_json = std.conversions.to_json;
>>> to_json(43)
"43"
>>> to_json(30000000000000000000000)
# "{\"$type\":\"number\",\"$sign\":1,\"$bytes\":[0,0,192,22,48,93,162,77,90,6]}"
```

## Composite

Maps and lists are mapped in their json equivalent:
```dices
#>>> let to_json = std.conversions.to_json;
>>> to_json([1,"a",3])
"[1,\"a\",3]"
>>> to_json(<|a:34, b:5|>)
# "{\"a\":34,\"b\":5}"
```

The main exceptions are maps with the `$type` keys, that are serialized in a `$content` tag:
```dices
#>>> let to_json = std.conversions.to_json;
>>> to_json(<|"$type": 42|>)
# "{\"$type\":\"map\",\"$content\":{\"$type\":42}}"
```

## Complex types

Both closures and intrisics can be serialized.

Closures body are serialized as binary data, while parameters and captures are serialized as json:
```dices
#>>> let to_json = std.conversions.to_json;
>>> to_json(|x| x+1)
# "{\"$type\":\"closure\",\"$params\":[\"x\"],\"$body\":[5,0,10,1,120,0,2,1,1]}"
>>> let captured = [1,2,3];
>>> to_json(|| captured)
# "{\"$type\":\"closure\",\"$params\":[],\"$captures\":{\"captured\":[1,2,3]},\"$body\":[10,8,99,97,112,116,117,114,101,100]}"
```

Intrisics instead are serialized with their internal name. Beware: this means that deserializing intrisics coming from another enviroment could result in errors, or strange behavior
```dices
#>>> let to_json = std.conversions.to_json;
>>> to_json(sum)
# "{\"$type\":\"intrisic\",\"$intrisic\":\"sum\"}"
```