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
        try {
            init.headers['Authorization'] = Http.getAuthHeader();
        } catch (e) {
            return new Result(null, e);
        }
        console.log('updatedInit', init);
        return fetch(path, init)
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

    private static getAuthHeader(): string {
        let publicKey = localStorage.getItem('public-key');
        let authToken = localStorage.getItem('auth-token');
        if (!publicKey || publicKey == '' || !authToken || authToken == '') {
            throw new Error('Failed to create auth header');
        }
        return 'Bearer ' + authToken + '$' + publicKey;
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
                        switch={this.state.switches.find(s => s.name == this.state.selectedSwitchName)}
                        back={() => this.setState({view: View.Switches, selectedSwitchName: null, selectedSwitchFlips: null})}
                    />)
                }
                </main>
            </div>
        );
    }
    getSwitchDetails(sw: Switch) {
        console.log('getting switch details', sw);
        this.setState({view: View.Loading});
        Http.put('/switch', sw, arr => arr.map(ScheduledFlip.fromJson))
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
    switch: Switch;
    back: () => void;
}

interface ISwitchInfoState {
    flips: ScheduledFlip[];
    switch: Switch;
    dirtyFlips: number[];
    switchDirty: boolean;
}

class SwitchInfo extends React.Component<ISwitchInfoProps, ISwitchInfoState> {
    constructor(props) {
        super(props);
        this.state = {
            switch: props.switch.clone(),
            flips: props.flips.map(f => f.clone()),
            dirtyFlips: [],
            switchDirty: false,
        };
    }
    render() {
        return (
            <div className="switch-info-container">
                <button
                    type="button"
                    className="back-button button"
                    onClick={() => this.props.back()}
                >Back</button>
                <h2 className="switch-name">{this.props.switch.name}</h2>
                <div className="direction-info">
                    <div className="direction-group">
                        <label>On Code</label>
                        <input
                            id="on-input"
                            defaultValue={this.props.switch.onCode.toString()}
                            type="number"
                            onChange={ev => this.updateSwitchCode(Direction.On, ev.currentTarget.value)}
                        />
                    </div>
                    <div className="direction-group">
                        <label>On Code</label>
                        <input
                            id="off-input"
                            defaultValue={this.props.switch.offCode.toString()}
                            type="number"
                            onChange={ev => this.updateSwitchCode(Direction.Off, ev.currentTarget.value)}
                        />
                    </div>
                </div>
                <div className="flips">
                {
                    this.state.flips.map((f, i) => {
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
                                            <th>K</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <tr>
                                            <td
                                                onClick={ev => this.updateFlip(i, WeekDay.Monday)}
                                            >{this.checkmark(f.dow.monday)}</td>
                                            <td
                                                onClick={ev => this.updateFlip(i, WeekDay.Tuesday)}
                                            >{this.checkmark(f.dow.tuesday)}</td>
                                            <td
                                                onClick={ev => this.updateFlip(i, WeekDay.Wednesday)}
                                            >{this.checkmark(f.dow.wednesday)}</td>
                                            <td
                                                onClick={ev => this.updateFlip(i, WeekDay.Thursday)}
                                            >{this.checkmark(f.dow.thursday)}</td>
                                            <td
                                                onClick={ev => this.updateFlip(i, WeekDay.Friday)}
                                            >{this.checkmark(f.dow.friday)}</td>
                                            <td
                                                onClick={ev => this.updateFlip(i, WeekDay.Saturday)}
                                            >{this.checkmark(f.dow.saturday)}</td>
                                            <td
                                                onClick={ev => this.updateFlip(i, WeekDay.Sunday)}
                                            >{this.checkmark(f.dow.sunday)}</td>
                                            <td
                                                onClick={() => this.nextKind(i)}
                                                title={this.flipKindTitle(f.kind)}
                                            >{this.flipKind(f.kind)}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        )
                    })
                }
                </div>
                <div className="switch-info-buttons">
                    <button
                        id="cancel-button"
                        className="cancel button"
                        onClick={() => this.cancel()}
                        >Cancel</button>
                        <button
                        id="save-button"
                        className="save button"
                        onClick={() => this.save()}
                    >Save</button>
                </div>
            </div>
        )
    }

    checkmark(b: boolean) {
        return b ? 'X' : ' ';
    }

    flipKind(kind: FlipKind) {
        switch (kind) {
            case FlipKind.Custom:
                return '🦹‍';
            case FlipKind.PreDawn:
                return '🔅';
            case FlipKind.Sunrise:
                return '🔆';
            case FlipKind.Dusk:
                return '🌝';
            case FlipKind.Sunset:
                return '🌚';
        }
    }

    flipKindTitle(kind: FlipKind) {
        switch (kind) {
            case FlipKind.Custom:
                return 'Custom';
            case FlipKind.PreDawn:
                return 'PreDawn';
            case FlipKind.Sunrise:
                return 'Sunrise';
            case FlipKind.Dusk:
                return 'Dusk';
            case FlipKind.Sunset:
                return 'Sunset';
        }
    }

    nextKind(idx: number) {
        let newFlip = this.state.flips[idx].clone();
        switch (newFlip.kind) {
            case FlipKind.Custom:
                newFlip.kind = FlipKind.PreDawn;
            break;
            case FlipKind.PreDawn:
                newFlip.kind = FlipKind.Sunrise;
            break;
            case FlipKind.Sunrise:
                newFlip.kind = FlipKind.Dusk;
            break;
            case FlipKind.Dusk:
                newFlip.kind = FlipKind.Sunset;
            break;
            case FlipKind.Sunset:
                newFlip.kind = FlipKind.Custom;
            break;
        }
        let newFlips = this.state.flips.map((f, i) => {
            if (i === idx) {
                return newFlip;
            }
            return f;
        });
        let dirtyFlips = this.state.dirtyFlips.map(i => i);
        if (dirtyFlips.indexOf(idx) < 0) {
            dirtyFlips.push(idx);
        }
        this.setState({flips: newFlips, dirtyFlips});
    }

    updateSwitchCode(direction: Direction, newCode: string) {
        let code: number;
        try {
            code = parseInt(newCode);
        } catch (e) {
            return console.error('Failed to parse code');
        }
        let newSwitch: Switch;
        if (direction === Direction.On) {
            newSwitch = new Switch(this.state.switch.id, this.state.switch.name, code, this.state.switch.offCode);
        } else {
            newSwitch = new Switch(this.state.switch.id, this.state.switch.name, this.state.switch.onCode, code);
        }
        this.setState(Object.assign({}, {switch: newSwitch, switchDirty: true}));
    }

    updateFlip(idx: number, day: WeekDay) {
        let flips = this.state.flips.map((f, i) => {
            if (i === idx) {
                let flip = f.clone();
                flip.dow.toggleDay(day);
                return flip;
            }
            return f;
        });
        let dirtyFlips = this.state.dirtyFlips.map(f => f);
        if (dirtyFlips.indexOf(idx) < 0) {
            dirtyFlips.push(idx);
        }
        this.setState({
            flips,
            dirtyFlips,
        });
    }

    async save() {
        let newState = {};
        if (this.state.switchDirty) {
            let res = await Http.post<Switch, Switch>('/update_switch', this.state.switch, Switch.fromJson);
            if (res.is_ok()) {
                newState['switchDirty'] = false;
                newState['switch'] = res.unwrap();
            } else {
                console.error(res.errorMessage());
            }
        }
        let saved = [];
        let newFlips = this.state.flips.map(f => f.clone());
        for (let idx of this.state.dirtyFlips.map(f => f)) {
            try {
                let res = await Http.post<ScheduledFlip, ScheduledFlip>('update_flip', this.state.flips[idx], ScheduledFlip.fromJson);
                if (res.is_ok()) {
                    saved.push(idx);
                    newFlips[idx] = res.unwrap();
                } else {
                    console.error(res.errorMessage());
                }
            } catch (e) {
                console.error('Error in dirtyFlips loop', e);
                continue;
            }
        }
        newState['flips'] = newFlips;
        let dirtyFlips = this.state.dirtyFlips.filter(f => saved.indexOf(f) < 0);
        newState['dirtyFlips'] = dirtyFlips;
        this.setState(newState);
    }
    cancel() {
        let switch_ = this.props.switch.clone();
        let flips = this.props.flips.map(f => f);
        let newState = {
            dirtyFlips: [],
            switchDirty: false,
            switch: switch_,
            flips,
        };
        this.setState(newState);
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
    clone(): Switch {
        return new Switch(
            this.id,
            this.name,
            this.onCode,
            this.offCode,
        );
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

    toJSON() {
        return {
            id: this.id,
            hour: this._hour,
            minute: this.minute,
            dow: this.dow,
            direction: this.flip,
            kind: this.kind,
        }
    }

    clone(): ScheduledFlip {
        return new ScheduledFlip(
            this.id,
            this._hour,
            this.minute,
            this.dow.clone(),
            this.flip,
            this.kind,
        )
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

    toggleDay(wd: WeekDay) {
        switch (wd) {
            case WeekDay.Monday:
                this.monday = !this.monday;
            break;
            case WeekDay.Tuesday:
                this.tuesday = !this.tuesday;
            break;
            case WeekDay.Wednesday:
                this.wednesday = !this.wednesday
            break;
            case WeekDay.Thursday:
                this.thursday = !this.thursday;
            break;
            case WeekDay.Friday:
                this.friday = !this.friday;
            break;
            case WeekDay.Saturday:
                this.saturday = !this.saturday;
            break;
            case WeekDay.Sunday:
                this.sunday = !this.sunday;
            break;
        }
    }

    clone(): DayOfTheWeek {
        return new DayOfTheWeek(
            this.monday,
            this.tuesday,
            this.wednesday,
            this.thursday,
            this.friday,
            this.saturday,
            this.sunday,
        )
    }
}

enum WeekDay {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
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