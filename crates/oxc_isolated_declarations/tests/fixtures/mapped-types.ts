import {K} from 'foo'
import {T} from 'bar'

export interface I {
	prop: {[key in K]: T}
}

export interface I2<K2 extends string> {
	prop: {[key in K2]: T}
}