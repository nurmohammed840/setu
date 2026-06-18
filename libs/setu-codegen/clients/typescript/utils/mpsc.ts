
export class MPSC<T> {
    #closed = false;
    #controller!: ReadableStreamDefaultController<T>;
    #writers: Array<[PromiseWithResolvers<void>, T]> = [];
    #readers: Array<ReadableStreamDefaultController<T>> = [];

    #wake_all_sender(reason?: any) {
        this.#closed = true;
        for (let [waker] of this.#writers) {
            waker.reject(reason)
        }
        this.#writers.length = 0;
    }

    public readonly stream: ReadableStream<T> = new ReadableStream<T>({
        start: c => this.#controller = c,
        cancel: reason => this.#wake_all_sender(reason),
        pull: async c => {
            let writer = this.#writers.shift();
            if (!writer) {
                this.#readers.push(c);
                return
            };
            let [waker, val] = writer;
            try {
                c.enqueue(val);
                waker.resolve();
            } catch (error) {
                waker.reject(error);
            }
        }
    });

    [Symbol.dispose]() {
        this.close()
    }

    send(val: T) {
        if (this.#closed) {
            throw new Error("Channel is closed");
        }
        let reader = this.#readers.shift();
        if (reader) {
            reader.enqueue(val);
            return Promise.resolve()
        }
        let waker = Promise.withResolvers<void>();
        this.#writers.push([waker, val]);
        return waker.promise;
    }

    close(reason?: any) {
        this.#wake_all_sender(reason);
        this.#controller.close();
    }

    error(reason?: any) {
        this.#wake_all_sender(reason);
        this.#controller.error(reason);
    }

    get isClosed() {
        return this.#closed
    }
}
