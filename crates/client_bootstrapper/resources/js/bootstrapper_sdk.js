class Bootstrapper extends EventTarget {
    _currentTask = undefined;

    constructor(startingTask) {
        super();

        this._currentTask = startingTask;
        setInterval(() => this._pollForTaskChanges(), 100);
    }

    getCurrentTask() {
        return this._currentTask;
    }

    /// Polls the Bootstrapper for task changes via the custom protocol.
    /// TODO: Use WebSockets for communication.
    _pollForTaskChanges() {
        console.debug("Polling for task changes");
        fetch("bootstrapper://server/current_task")
            .then((response) => response.headers.get("x-current-task"))
            .then((task) => {
                if (task && task !== this._currentTask) {
                    console.debug("Task changed to: " + task);
    
                    this._currentTask = task;
                    this.dispatchEvent(new Event("NewTask"))
                }
            });
    }
}
