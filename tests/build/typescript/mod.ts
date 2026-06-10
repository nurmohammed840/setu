// AUTO-GENERATED FILE. DO NOT EDIT.
import * as $ from "./lib/mod.ts";
export const $etu = { RPC: $.RPC };

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
