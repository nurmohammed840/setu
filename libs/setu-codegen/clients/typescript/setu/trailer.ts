import { Status } from "../status.ts";
import { FrameHeader } from "./frame.ts";

export class Trailer {
    static OK_ENCODED = [FrameHeader.new({ lenSize: 1, trailer: Status.Ok }).encode(), 0];
}