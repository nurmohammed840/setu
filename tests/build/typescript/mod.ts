// AUTO-GENERATED FILE. DO NOT EDIT.
import * as $ from "./lib/mod.ts";
export const $etu = { RPC: $.RPC };
const $E = {
	Data: function Struct(this: $.lipi.Encode, z: Data) {
		$.lipi.StructEncoder(this, [
			[1, z.u8, this.U8],
			[2, z.u16, this.U16],
			[3, z.u32, this.U32],
			[4, z.u64, this.U64],
			[5, z.i8, this.I8],
			[6, z.i16, this.I16],
			[7, z.i32, this.I32],
			[8, z.i64, this.I64],
			[9, z.f32, this.F32],
			[10, z.f64, this.F64],
			[11, z.bool, this.Bool],
			[12, z.string, this.Str],
			[13, z.numeric, $E.Numerical],
		]);
	},
	HelloRequest: function Struct(this: $.lipi.Encode, z: HelloRequest) {
		$.lipi.StructEncoder(this, [
			[1, z.name, this.Str],
		]);
	},
}
const $D = {
	Data: function Struct(this: $.lipi.Decode): Data {
		return $.lipi.StructDecoder(this, [
			[1, "u8", this.U8, true],
			[2, "u16", this.U16, true],
			[3, "u32", this.U32, true],
			[4, "u64", this.U64, true],
			[5, "i8", this.I8, true],
			[6, "i16", this.I16, true],
			[7, "i32", this.I32, true],
			[8, "i64", this.I64, true],
			[9, "f32", this.F32, true],
			[10, "f64", this.F64, true],
			[11, "bool", this.Bool, true],
			[12, "string", this.Str, true],
			[13, "numeric", $D.Numerical, true],
		]);
	},
	HelloReply: function Struct(this: $.lipi.Decode): HelloReply {
		return $.lipi.StructDecoder(this, [
			[1, "message", this.Str, true],
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

export interface say_hello {
	input: HelloRequest,
}
export function say_hello(z: say_hello, ctx: $.Context = {}) {
	return $.rpc(
		1, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.input, $E.HelloRequest],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, $D.HelloReply, true);
		},
	);
}

export interface add {
	a: number,
	b: number,
}
export function add(z: add, ctx: $.Context = {}) {
	return $.rpc(
		2, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.a, this.I32],
				[1, z.b, this.I32],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, this.I32, true);
		},
	);
}

export interface find_in_string {
	input: string,
	pat: string,
}
export function find_in_string(z: find_in_string, ctx: $.Context = {}) {
	return $.rpc(
		3, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.input, this.Str],
				[1, z.pat, this.Str],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, this.U32, false);
		},
	);
}

export interface print {
	msg: string,
}
export function print(z: print, ctx: $.Context = {}) {
	return $.rpc(
		4, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.msg, this.Str],
			]);
		},
		function() {}
	);
}

export interface store {
	msg: string,
}
export function store(z: store, ctx: $.Context = {}) {
	return $.rpc(
		5, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.msg, this.Str],
			]);
		},
		function() {}
	);
}

export function load(ctx: $.Context = {}) {
	return $.rpc(
		6, ctx,
		function() {
			$.lipi.StructEncoder(this, []);
		},
		function() {
			return $.lipi.OutputDecoder(this, this.Str, false);
		},
	);
}

export function what_is_my_ip(ctx: $.Context = {}) {
	return $.rpc(
		7, ctx,
		function() {
			$.lipi.StructEncoder(this, []);
		},
		function() {
			return $.lipi.OutputDecoder(this, this.Str, true);
		},
	);
}

export interface fetch_user_ids {
	count: number,
}
export function fetch_user_ids(z: fetch_user_ids, ctx: $.Context = {}) {
	return $.sse(
		8, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.count, this.U8],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, this.U8, true);
		},
		function() {
			return $.lipi.OutputDecoder(this, this.Str, true);
		},
	);
}

export function random_data(ctx: $.Context = {}) {
	return $.rpc(
		101, ctx,
		function() {
			$.lipi.StructEncoder(this, []);
		},
		function() {
			return $.lipi.OutputDecoder(this, $D.Data, true);
		},
	);
}

export interface echo_data {
	input: Data,
}
export function echo_data(z: echo_data, ctx: $.Context = {}) {
	return $.rpc(
		102, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.input, $E.Data],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, $D.Data, true);
		},
	);
}

export interface compare_data {
	left: Data,
	right: Data,
}
export function compare_data(z: compare_data, ctx: $.Context = {}) {
	return $.rpc(
		103, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.left, $E.Data],
				[1, z.right, $E.Data],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, this.Bool, true);
		},
	);
}

export function random_js_value(ctx: $.Context = {}) {
	return $.rpc(
		104, ctx,
		function() {
			$.lipi.StructEncoder(this, []);
		},
		function() {
			return $.lipi.OutputDecoder(this, $D.JsValue, true);
		},
	);
}

export interface echo_js_value {
	input: JsValue,
}
export function echo_js_value(z: echo_js_value, ctx: $.Context = {}) {
	return $.rpc(
		105, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.input, $E.JsValue],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, $D.JsValue, true);
		},
	);
}

export interface compare_js_value {
	left: JsValue,
	right: JsValue,
}
export function compare_js_value(z: compare_js_value, ctx: $.Context = {}) {
	return $.rpc(
		106, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, z.left, $E.JsValue],
				[1, z.right, $E.JsValue],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, this.Bool, true);
		},
	);
}
