// AUTO-GENERATED FILE. DO NOT EDIT.
import * as $ from "./lib/mod.ts";
export const $etu = { RPC: $.RPC };

export class HelloReply {
	message: string;
	constructor(args: HelloReply) {
		this.message = args.message;
	}
	static decoder = function Struct(this: $.lipi.Decode) {
		return new HelloReply($.lipi.StructDecoder(this, [
			["message", 1, this.Str, true],
		]));
	}
}

export class HelloRequest {
	name: string;
	constructor(args: HelloRequest) {
		this.name = args.name;
	}
	static encoder = function Struct(this: $.lipi.Encode, args: HelloRequest) {
		$.lipi.StructEncoder(this, [
			[1, args.name, this.Str],
		]);
	}
}

export interface say_hello {
	req: HelloRequest,
}
export function say_hello(args: say_hello, ctx: $.Context = {}) {
	let [i, o] = $.rpc(1, ctx, function () {
		return $.lipi.OutputDecoder(this, HelloReply.decoder, true);
	});
	i.sendAndClose(function (this: $.lipi.Encode) {
		$.lipi.StructEncoder(this, [
			[0, args.req, HelloRequest.encoder],
		]);
	});
	return o;
}
