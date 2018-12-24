import * as React from 'react';
import * as ReactDom from 'react-dom';
import * as moment from 'moment';

interface IAppState {
    switches: Switch[];
    view: View;
    selectedSwitchFlips?: ScheduledFlip[];
    selectedSwitchName?: string;
}

enum View {
    Loading,
    Switches,
    SwitchDetail,
}

class Http {
    static async get<T>(path: string, mapper?: (j: any) => T): Promise<Result<T>> {
        console.log(`Http.get(${path})`);
        return Http.send(path, {method: 'GET'}, mapper);
    }

    static async post<T, U>(path: string, body?: T, mapper?: (j: any) => U): Promise<Result<U>> {
        return Http.sendWithBody(path, 'POST', body, mapper);
    }

    static async put<T, U>(path: string, body?: T, mapper?: (j: any) => U): Promise<Result<U>> {
        return Http.sendWithBody(path, 'PUT', body, mapper);
    }

    static async delete<T, U>(path: string, body?: T, mapper?: (j: any) => U): Promise<Result<U>> {
        return Http.sendWithBody(path, 'DELETE', body, mapper);
    }

    private static async sendWithBody<T, U>(path: string, method: string, body?: T, mapper?: (j: any) => U): Promise<Result<U>> {
        let init: RequestInit = {method};
        if (body) {
            init = Object.assign(init, {
                body: JSON.stringify(body),
                headers: {
                    'Content-Type': 'application/json',
                }
            });
        }
        return Http.send(path, init, mapper);
    }

    private static async send<T>(path: string, init: RequestInit, mapper: (j: any) => T): Promise<Result<T>> {
        console.log(`Http.send(${path}, ${init})`);
        if (!init.headers) {
            init.headers = {};
        }
        let updatedInit = Object.assign({ headers: {
            Authorization: `Bearer ${localStorage.getItem('bearer-token')}}`
            }
        }, init);
        console.log('updatedInit', updatedInit);
        return fetch(path, updatedInit)
            .then(res => {
                console.log('got response, parsing json');
                return res.json();
            })
            .then(json => {
                console.log('got response with json');
                if (json.message) {
                    console.log('response was an error');
                    return Result.Err(new Error(json.message));
                } else {
                    if (mapper) {
                        try {
                            let mapped = mapper(json);
                            return Result.Ok(mapped);
                        } catch (e) {
                            return Result.Err(e);
                        }
                    } else {
                        return Result.Ok(json)
                    }
                }
            })
            .catch(e => {
                console.error('Error sending http request', e);
                return Result.Err(e) as Result<T>;
            });
    }
}

class Result<T> {
    constructor(
        private success: T = null,
        private failure: Error = null,
    ) {
        if (!success && !failure) {
            throw new Error('Cannot construct empty result');
        }
    }

    static Ok<T>(value: T): Result<T> {
        return new Result<T>(value, null);
    }

    static Err<T>(err: Error): Result<T> {
        return new Result(null, err);
    }

    is_ok(): boolean {
        return this.success != null;
    }

    is_err(): boolean {
        return this.success == null;
    }

    unwrap(): T {
        if (!this.is_ok()) {
            throw this.failure;
        }
        return this.success;
    }

    unwrapOr(value: T): T {
        if (!this.is_ok()) {
            return value;
        }
        return this.success
    }
    unwrapOrElse(f: (err: Error) => T) {
        if (!this.is_ok()) {
            return f(this.failure)
        }
        return this.success;
    }
    errorMessage() {
        if (!this.is_ok()) {
            return this.failure.message;
        }
        throw new Error(`Attempt to access error message of successful result ${this}`);
    }
}

export class App extends React.Component<{}, IAppState> {
    constructor(props) {
        super(props);
        this.state = {
            switches: [],
            view: View.Loading,
        };
    }
    componentDidMount() {
        console.log('componentDidMount');
        Http.get('/switches', (arr) => arr.map(Switch.fromJson))
            .then((res: Result<Switch[]>) => {
                console.log('got switches', res);
                let switches = res.unwrapOr([]);
                this.setState({
                    switches,
                    view: View.Switches,
                });
            })
    }

    render() {
        return (
            <div>
                <header>
                    <h1>Robohome Light Switch</h1>
                </header>
                <main>
                {
                    this.state.view === View.Loading
                    ? (<LoadingSpinner
                    />)
                    : this.state.view === View.Switches
                    ? (<SwitchList
                        switches={this.state.switches}
                        infoHandler={(sw) => this.getSwitchDetails(sw)}
                    />)
                    : (<SwitchInfo
                        flips={this.state.selectedSwitchFlips}
                        name={this.state.selectedSwitchName}
                        onCode={this.state.switches.find(s => s.name == this.state.selectedSwitchName).onCode}
                        offCode={this.state.switches.find(s => s.name == this.state.selectedSwitchName).offCode}
                    />)
                }
                </main>
            </div>
        );
    }
    getSwitchDetails(sw: Switch) {
        console.log('getting switch details', sw);
        this.setState({view: View.Loading});
        Http.post('/switch', sw, arr => arr.map(ScheduledFlip.fromJson))
            .then(r => {
                if (r.is_err()) {
                    return this.httpFail(r.errorMessage());
                }
                let flips = r.unwrapOr([]);
            this.setState({
                view: View.SwitchDetail,
                selectedSwitchFlips: flips,
                selectedSwitchName: sw.name,
            });
        });
    }

    httpFail(msg) {
        console.error('httpFail', msg);
        this.setState({
            view: View.Switches,
        });
    }
}

class LoadingSpinner extends React.Component<{}, {}> {
    render() {
        return (
            <div className="loading-spinner">
                <div className="spinner-inner"/>
                <div className="spinner-inner"/>
            </div>
        );
    }
}

interface ISwitchListProps {
    switches: Switch[];
    infoHandler: (s: Switch) => void;
}

class SwitchList extends React.Component<ISwitchListProps, {}> {
    render() {
        return (
            <div className="switch-list">
                {
                    this.props.switches.map((s, i) => {
                        return (
                            <SwitchPlate
                                switchInfo={s} key={`switch-plate=${i}`}
                                infoHandler={() => this.props.infoHandler(s)}
                            />
                        );
                    })
                }
            </div>
        );
    }
}

interface ISwitchPlateProps {
    switchInfo: Switch;
    infoHandler: () => void;
}

class SwitchPlate extends React.Component<ISwitchPlateProps, {}> {
    render() {
        return (
            <div className="switch-plate">
                <span className="switch-name">{this.props.switchInfo.name}</span>
                <div
                    className="button on"
                    title="turn on light"
                    onClick={ev => this.flipSwitch(Direction.On, ev)}
                >
                    <span>On</span>
                </div>
                <div
                    className="button off"
                    title="turn on light"
                    onClick={ev => this.flipSwitch(Direction.Off, ev)}
                >
                    <span>Off</span>
                </div>
                <div className="button info" title="get switch details"
                onClick={this.props.infoHandler.bind(this)}
                >
                    <span>i</span>
                </div>
            </div>
        );
    }

    flipSwitch(direction: Direction, ev: React.MouseEvent<HTMLDivElement>) {
        let code: number;
        switch (direction) {
            case Direction.On:
                    code = this.props.switchInfo.onCode;
                break;
            case Direction.Off:
                    code = this.props.switchInfo.offCode;
                break;
        }
        Http.post('/flip', new Flip(-1, -1, code))
            .then(res => {
                if (res.is_ok()) {
                    console.log('flipped: ', res.unwrap());
                } else {
                    console.error(res.errorMessage());
                }
            })
            .catch(e => console.error('Unable to post flip', e));
    }
}

interface ISwitchInfoProps {
    flips: ScheduledFlip[];
    name: string;
    onCode: number,
    offCode: number,
}

class SwitchInfo extends React.Component<ISwitchInfoProps, {}> {
    render() {
        return (
            <div className="switch-info-container">
                <h2 className="switch-name">{this.props.name}</h2>
                <div className="direction-info">
                    <div className="direction-group">
                        <label>On Code</label>
                        <input
                            id="on-input"
                            defaultValue={this.props.onCode.toString()}
                            type="number"
                        />
                    </div>
                    <div className="direction-group">
                        <label>On Code</label>
                        <input
                            id="off-input"
                            defaultValue={this.props.offCode.toString()}
                            type="number"
                        />
                    </div>
                </div>
                <div className="flips">
                {
                    this.props.flips.map((f, i) => {
                        return (
                            <div className="flip-info" key={`flip-info-${i}`}>
                                <span>{f.hour}</span>:<span>{`0${f.minute}`.substr(-2)}</span><span>{f.tod}</span>
                                <table>
                                    <thead>
                                        <tr>
                                            <th>M</th>
                                            <th>T</th>
                                            <th>W</th>
                                            <th>R</th>
                                            <th>F</th>
                                            <th>S</th>
                                            <th>U</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <tr>
                                            <td>{this.checkmark(f.dow.monday)}</td>
                                            <td>{this.checkmark(f.dow.tuesday)}</td>
                                            <td>{this.checkmark(f.dow.wednesday)}</td>
                                            <td>{this.checkmark(f.dow.thursday)}</td>
                                            <td>{this.checkmark(f.dow.friday)}</td>
                                            <td>{this.checkmark(f.dow.saturday)}</td>
                                            <td>{this.checkmark(f.dow.sunday)}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        )
                    })
                }
                </div>
                <div className="switch-info-buttons">
                    <button id="cancel-button" className="cancel"

                    >Cancel</button>
                    <button id="save-button" className="save">Save</button>
                </div>
            </div>
        )
    }

    checkmark(b) {
        return b ? 'X' : ' ';
    }
}

class Switch {
    constructor(
        public id: number,
        public name: string,
        public onCode: number,
        public offCode: number,
    ) {}
    static fromJson(json: any): Switch {
        return new Switch(
            json.id,
            json.name,
            json.onCode,
            json.offCode,
        )
    }
}

function getLocalOffset() {
    let offsetMinutes = moment().utcOffset();
    return {
        hours: Math.floor(offsetMinutes / 60),
        minutes: Math.floor(offsetMinutes % 60),
    }
}
class ScheduledFlip {
    private static offsetHours;
    private static offsetMinute;
    private _hour: number;
    public minute: number;
    constructor(
        public id: number,
        hour: number,
        minute: number,
        public dow: DayOfTheWeek,
        public flip: Direction,
        public kind: FlipKind,
    ) {
        if (!ScheduledFlip.offsetHours || !ScheduledFlip.offsetMinute) {
            let {hours, minutes} = getLocalOffset();
            ScheduledFlip.offsetHours = hours;
            ScheduledFlip.offsetMinute = minutes;
        }
        this._hour = hour;// + ScheduledFlip.offsetHours;
        this.minute = minute;// + ScheduledFlip.offsetMinute;
    }

    get hour() {
        if (this._hour > 12) {
            return this._hour - 12;
        }
        return this._hour;
    }

    get tod() {
        return this._hour > 12 ? 'p' : 'a';
    }

    static fromJson(json) {
        return new ScheduledFlip(
            json.id,
            json.hour,
            json.minute,
            DayOfTheWeek.fromJson(json.dow),
            json.direction,
            json.kind,
        );
    }
}

class Flip {
    constructor(
        public hour: number,
        public minute: number,
        public code: number,
    ) { }
}

class DayOfTheWeek {
    constructor(
        public monday: boolean,
        public tuesday: boolean,
        public wednesday: boolean,
        public thursday: boolean,
        public friday: boolean,
        public saturday: boolean,
        public sunday: boolean,
    ) { }

    static fromJson(json) {
        return new DayOfTheWeek(
            json.monday,
            json.tuesday,
            json.wednesday,
            json.thursday,
            json.friday,
            json.saturday,
            json.sunday,
        )
    }
}

enum Direction {
    On = 'On',
    Off = 'Off',
}
enum FlipKind {
    Custom = 'Custom',
    PreDawn = 'PreDawn',
    Sunrise = 'Sunrise',
    Dusk = 'Dusk',
    Sunset = 'Sunset',
}
const mockSwitches = [
    new Switch(1, 'Living Room', 44444, 22222),
    new Switch(1, 'Xmas Tree', 55555, 11111)
]

ReactDom.render(<App />, document.getElementById('app'));