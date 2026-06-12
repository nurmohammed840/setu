// AUTO-GENERATED FILE. DO NOT EDIT.
import * as $ from "./lib/mod.ts";
export const $etu = { RPC: $.RPC };

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
}
namespace Data {
	export const encoder = function Struct(this: $.lipi.Encode, args: Data) {
		$.lipi.StructEncoder(this, [
			[1, args.u8, this.U8],
			[2, args.u16, this.U16],
			[3, args.u32, this.U32],
			[4, args.u64, this.U64],
			[5, args.i8, this.I8],
			[6, args.i16, this.I16],
			[7, args.i32, this.I32],
			[8, args.i64, this.I64],
			[9, args.f32, this.F32],
			[10, args.f64, this.F64],
			[11, args.bool, this.Bool],
			[12, args.string, this.Str],
		]);
	}
	export const decoder = function Struct(this: $.lipi.Decode): Data {
		return $.lipi.StructDecoder(this, [
			["u8", 1, this.U8, true],
			["u16", 2, this.U16, true],
			["u32", 3, this.U32, true],
			["u64", 4, this.U64, true],
			["i8", 5, this.I8, true],
			["i16", 6, this.I16, true],
			["i32", 7, this.I32, true],
			["i64", 8, this.I64, true],
			["f32", 9, this.F32, true],
			["f64", 10, this.F64, true],
			["bool", 11, this.Bool, true],
			["string", 12, this.Str, true],
		]);
	}
}

export interface HelloReply {
	message: string;
}
namespace HelloReply {
	export const decoder = function Struct(this: $.lipi.Decode): HelloReply {
		return $.lipi.StructDecoder(this, [
			["message", 1, this.Str, true],
		]);
	}
}

export interface HelloRequest {
	name: string;
}
namespace HelloRequest {
	export const encoder = function Struct(this: $.lipi.Encode, args: HelloRequest) {
		$.lipi.StructEncoder(this, [
			[1, args.name, this.Str],
		]);
	}
}

export interface say_hello {
	input: HelloRequest,
}
export function say_hello(args: say_hello, ctx: $.Context = {}) {
	return $.rpc(
		1, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, args.input, HelloRequest.encoder],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, HelloReply.decoder, true);
		},
	);
}

export interface add {
	a: number,
	b: number,
}
export function add(args: add, ctx: $.Context = {}) {
	return $.rpc(
		2, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, args.a, this.I32],
				[1, args.b, this.I32],
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
export function find_in_string(args: find_in_string, ctx: $.Context = {}) {
	return $.rpc(
		3, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, args.input, this.Str],
				[1, args.pat, this.Str],
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
export function print(args: print, ctx: $.Context = {}) {
	return $.rpc(
		4, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, args.msg, this.Str],
			]);
		},
		function() {}
	);
}

export interface store {
	msg: string,
}
export function store(args: store, ctx: $.Context = {}) {
	return $.rpc(
		5, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, args.msg, this.Str],
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
export function fetch_user_ids(args: fetch_user_ids, ctx: $.Context = {}) {
	return $.sse(
		8, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, args.count, this.U8],
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
			return $.lipi.OutputDecoder(this, Data.decoder, true);
		},
	);
}

export interface echo_data {
	input: Data,
}
export function echo_data(args: echo_data, ctx: $.Context = {}) {
	return $.rpc(
		102, ctx,
		function() {
			$.lipi.StructEncoder(this, [
				[0, args.input, Data.encoder],
			]);
		},
		function () {
			return $.lipi.OutputDecoder(this, Data.decoder, true);
		},
	);
}

export interface compare_data {
	left: Data,
	right: Data,
}
export function compare_data(args: compare_data, ctx: $.Context = {}) {
	return $.rpc(
		103, ctx,
		function () {
			$.lipi.StructEncoder(this, [
				[0, args.left, Data.encoder],
				[1, args.right, Data.encoder],
			]);
		},
		function() {
			return $.lipi.OutputDecoder(this, this.Bool, true);
		},
	);
}
