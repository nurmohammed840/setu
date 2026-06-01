import { assert } from "./common.ts";

export class MPSC<T> {
    #controller!: ReadableStreamDefaultController<T>;
    #closed = false;

    public readonly stream: ReadableStream<T> = new ReadableStream<T>({
        start: (controller) => {
            this.#controller = controller;
        },
        cancel: () => {
            this.#closed = true;
        }
    });

    [Symbol.dispose]() {
        this.close()
    }

    send(val: T) {
        assert(!this.#closed, Error, "Channel is closed");
        this.#controller.enqueue(val);
    }

    close() {
        this.#closed = true;
        this.#controller.close()
    }

    get isClosed() {
        return this.#closed
    }
}
