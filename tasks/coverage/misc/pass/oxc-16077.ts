// Valid parameter properties with simple identifiers
class ValidA {constructor(private x){}}
class ValidB {constructor(public y){}}
class ValidC {constructor(protected z){}}
class ValidD {constructor(readonly w){}}
class ValidF {constructor(private x = 1){}}
class ValidG {constructor(public y: number = 2){}}
