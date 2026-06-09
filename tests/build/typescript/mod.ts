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
	let [i, o] = $.rpc(1, ctx, function () {
		return $.lipi.OutputDecoder(this, HelloReply.decoder, true);
	});
	i.sendAndClose(function (this: $.lipi.Encode) {
		$.lipi.StructEncoder(this, [
			[0, args.input, HelloRequest.encoder],
		]);
	});
	return o;
}

export interface add {
	a: number,
	b: number,
}
export function add(args: add, ctx: $.Context = {}) {
	let [i, o] = $.rpc(2, ctx, function () {
		return $.lipi.OutputDecoder(this, this.I32, true);
	});
	i.sendAndClose(function (this: $.lipi.Encode) {
		$.lipi.StructEncoder(this, [
			[0, args.a, this.I32],
			[1, args.b, this.I32],
		]);
	});
	return o;
}

export interface find_in_string {
	input: string,
	pat: string,
}
export function find_in_string(args: find_in_string, ctx: $.Context = {}) {
	let [i, o] = $.rpc(3, ctx, function () {
		return $.lipi.OutputDecoder(this, this.U32, false);
	});
	i.sendAndClose(function (this: $.lipi.Encode) {
		$.lipi.StructEncoder(this, [
			[0, args.input, this.Str],
			[1, args.pat, this.Str],
		]);
	});
	return o;
}

export interface print {
	msg: string,
}
export function print(args: print, ctx: $.Context = {}) {
	let [i, o] = $.rpc(4, ctx, function () {
	});
	i.sendAndClose(function (this: $.lipi.Encode) {
		$.lipi.StructEncoder(this, [
			[0, args.msg, this.Str],
		]);
	});
	return o;
}

export interface store {
	msg: string,
}
export function store(args: store, ctx: $.Context = {}) {
	let [i, o] = $.rpc(5, ctx, function () {
	});
	i.sendAndClose(function (this: $.lipi.Encode) {
		$.lipi.StructEncoder(this, [
			[0, args.msg, this.Str],
		]);
	});
	return o;
}

export function load(ctx: $.Context = {}) {
	let [i, o] = $.rpc(6, ctx, function () {
		return $.lipi.OutputDecoder(this, this.Str, false);
	});
	i.sendAndClose(function (this: $.lipi.Encode) {
		$.lipi.StructEncoder(this, [
		]);
	});
	return o;
}

export function what_is_my_ip(ctx: $.Context = {}) {
	let [i, o] = $.rpc(7, ctx, function () {
		return $.lipi.OutputDecoder(this, this.Str, true);
	});
	i.sendAndClose(function (this: $.lipi.Encode) {
		$.lipi.StructEncoder(this, [
		]);
	});
	return o;
}

export interface fetch_user_ids {
	count: number,
}
export function fetch_user_ids(args: fetch_user_ids, ctx: $.Context = {}) {
}
