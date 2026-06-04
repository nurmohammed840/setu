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
