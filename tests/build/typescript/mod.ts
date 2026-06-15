// AUTO-GENERATED FILE. DO NOT EDIT.
import * as $ from "./lib/mod.ts";
export const $etu = { RPC: $.RPC };

const $FE = $.lipi.FieldEncoder;
const $SE = $.lipi.StructEncoder;
const $SD = $.lipi.StructDecoder;
const $OD = $.lipi.OutputDecoder;
const $ED = $.lipi.EnumDecoder;

const $E = {
	Data: function Struct(this: $.lipi.Encode, z: Data) {
		let _ = this;
		$SE(_, [
			[1, z.u8, _.U8],
			[2, z.u16, _.U16],
			[3, z.u32, _.U32],
			[4, z.u64, _.U64],
			[5, z.i8, _.I8],
			[6, z.i16, _.I16],
			[7, z.i32, _.I32],
			[8, z.i64, _.I64],
			[9, z.f32, _.F32],
			[10, z.f64, _.F64],
			[11, z.bool, _.Bool],
			[12, z.string, _.Str],
			[13, z.numeric, $E.Numerical],
		]);
	},
	JsValue: function Union(this: $.lipi.Encode, z: JsValue) {
		let _ = this;
		switch (z.type) {
			case "Null": return $FE(_, [0, false, _.Bool]);
			case "Bool": return $FE(_, [1, z.value, _.Bool]);
			case "Number": return $FE(_, [2, z.value, _.F64]);
			case "String": return $FE(_, [3, z.value, _.Str]);
			case "Array": return $FE(_, [4, z.value, _.List($E.JsValue)]);
			case "Object": return $FE(_, [5, z.value, _.Table(_.Str, $E.JsValue)]);
		}
	},
	Numerical: function U8(this: $.lipi.Encode, z: Numerical) {
		this.U8(z)
	},
	HelloRequest: function Struct(this: $.lipi.Encode, z: HelloRequest) {
		let _ = this;
		$SE(_, [
			[1, z.name, _.Str],
		]);
	},
}
const $D = {
	Data: function Struct(this: $.lipi.Decode): Data {
		let _ = this;
		return $SD(_, [
			[1, "u8", _.U8, 1],
			[2, "u16", _.U16, 1],
			[3, "u32", _.U32, 1],
			[4, "u64", _.U64, 1],
			[5, "i8", _.I8, 1],
			[6, "i16", _.I16, 1],
			[7, "i32", _.I32, 1],
			[8, "i64", _.I64, 1],
			[9, "f32", _.F32, 1],
			[10, "f64", _.F64, 1],
			[11, "bool", _.Bool, 1],
			[12, "string", _.Str, 1],
			[13, "numeric", $D.Numerical, 1],
		]);
	},
	JsValue: function Struct(this: $.lipi.Decode): JsValue {
		let _ = this;
		return $ED(_, [
			[0, "Null", _.Bool, 0],
			[1, "Bool", _.Bool, 1],
			[2, "Number", _.F64, 1],
			[3, "String", _.Str, 1],
			[4, "Array", _.List($D.JsValue), 1],
			[5, "Object", _.Table(_.Str, $D.JsValue), 1],
		]);
	},
	Numerical: function U8(this: $.lipi.Decode): Numerical {
		let tag = this.U8();
		switch (tag) {
			case 1: return Numerical.A;
			case 2: return Numerical.B;
			case 3: return Numerical.C;
			default: throw new Error(`unknown tag: ${tag}`);
		}
	},
	HelloReply: function Struct(this: $.lipi.Decode): HelloReply {
		let _ = this;
		return $SD(_, [
			[1, "message", _.Str, 1],
		]);
	},
}
export interface Data {
	u8: number;
	u16: number;
	u32: number;
	u64: bigint;
	i8: number;
	i16: number;
	i32: number;
	i64: bigint;
	f32: number;
	f64: number;
	bool: boolean;
	string: string;
	numeric: Numerical;
}
export type JsValue =
	| { type: "Null" }
	| { type: "Bool"; value: boolean }
	| { type: "Number"; value: number }
	| { type: "String"; value: string }
	| { type: "Array"; value: Array<JsValue> }
	| { type: "Object"; value: Map<string, JsValue> }

export enum Numerical {
	A = 1,
	B = 2,
	C = 3,
}
export interface HelloReply {
	message: string;
}
export interface HelloRequest {
	name: string;
}

export function say_hello(input: HelloRequest, ctx: $.Context = {}) {
	return $.rpc(
		1, ctx,
		_ => $SE(_, [[0, input, $E.HelloRequest]]),
		_ => $OD(_, $D.HelloReply, true),
	);
}

export interface add {
	a: number,
	b: number,
}
export function add(z: add, ctx: $.Context = {}) {
	return $.rpc(
		2, ctx,
		_ => $SE(_, [
			[0, z.a, _.I32],
			[1, z.b, _.I32],
		]),
		_ => $OD(_, _.I32, true),
	);
}

export interface find_in_string {
	input: string,
	pat: string,
}
export function find_in_string(z: find_in_string, ctx: $.Context = {}) {
	return $.rpc(
		3, ctx,
		_ => $SE(_, [
			[0, z.input, _.Str],
			[1, z.pat, _.Str],
		]),
		_ => $OD(_, _.U32, false),
	);
}

export function print(msg: string, ctx: $.Context = {}) {
	return $.rpc(
		4, ctx,
		_ => $SE(_, [[0, msg, _.Str]]),
		_ => {}
	);
}

export function store(msg: string, ctx: $.Context = {}) {
	return $.rpc(
		5, ctx,
		_ => $SE(_, [[0, msg, _.Str]]),
		_ => {}
	);
}

export function load(ctx: $.Context = {}) {
	return $.rpc(
		6, ctx,
		_ => $SE(_, []),
		_ => $OD(_, _.Str, false),
	);
}

export function what_is_my_ip(ctx: $.Context = {}) {
	return $.rpc(
		7, ctx,
		_ => $SE(_, []),
		_ => $OD(_, _.Str, true),
	);
}

export function fetch_user_ids(count: number, ctx: $.Context = {}) {
	return $.sse(
		8, ctx,
		_ => $SE(_, [[0, count, _.U8]]),
		_ => $OD(_, _.U8, true),
		_ => $OD(_, _.Str, true),
	);
}

export function random_data(ctx: $.Context = {}) {
	return $.rpc(
		101, ctx,
		_ => $SE(_, []),
		_ => $OD(_, $D.Data, true),
	);
}

export function echo_data(input: Data, ctx: $.Context = {}) {
	return $.rpc(
		102, ctx,
		_ => $SE(_, [[0, input, $E.Data]]),
		_ => $OD(_, $D.Data, true),
	);
}

export interface compare_data {
	left: Data,
	right: Data,
}
export function compare_data(z: compare_data, ctx: $.Context = {}) {
	return $.rpc(
		103, ctx,
		_ => $SE(_, [
			[0, z.left, $E.Data],
			[1, z.right, $E.Data],
		]),
		_ => $OD(_, _.Bool, true),
	);
}

export function random_js_value(ctx: $.Context = {}) {
	return $.rpc(
		104, ctx,
		_ => $SE(_, []),
		_ => $OD(_, $D.JsValue, true),
	);
}

export function echo_js_value(input: JsValue, ctx: $.Context = {}) {
	return $.rpc(
		105, ctx,
		_ => $SE(_, [[0, input, $E.JsValue]]),
		_ => $OD(_, $D.JsValue, true),
	);
}

export interface compare_js_value {
	left: JsValue,
	right: JsValue,
}
export function compare_js_value(z: compare_js_value, ctx: $.Context = {}) {
	return $.rpc(
		106, ctx,
		_ => $SE(_, [
			[0, z.left, $E.JsValue],
			[1, z.right, $E.JsValue],
		]),
		_ => $OD(_, _.Bool, true),
	);
}
