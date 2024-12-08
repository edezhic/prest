/**  @jsx h @jsxFrag Fragment */
import { h, render, Fragment } from "./admin/preact.js";

type Level = "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR";
type Attrs = { [key: string]: string };
type TargetFilter = string | null;

type Trace = {
    timestamp: string,
    level: Level,
    target: string,
    fields: Attrs,
    span?: Attrs,
    spans?: [Attrs],
};

let traces: Trace[] = [];
let container: HTMLElement | null = null;
let index = 0;
let levelFilter: Level = "TRACE";
let targetFilter: TargetFilter = null;

const PER_PAGE = 10;
const DT_OPTIONS: Intl.DateTimeFormatOptions = {
    hour: "numeric",
    minute: "numeric",
    second: "numeric",
    fractionalSecondDigits: 2,
}

async function loadTraces(period: string) {
    container = document.getElementById("traces-container");
    const resp = await fetch(`/admin/traces/${period}`);
    const rawText = await resp.text();
    const text = `[${rawText.trimEnd().slice(0, -1)}]`;
    traces = JSON.parse(text).reverse();
    index = 0;
    levelFilter = "TRACE";
    targetFilter = null;
    renderTraces();
}

const renderTraces = () => render(Traces(), document.getElementById('traces-container'));

const setLevelFilter = (value: Level) => {
    levelFilter = value;
    index = 0;
    renderTraces();
}
const setTargetFilter = (value: string) => {
    targetFilter = value === "" ? null : value;
    index = 0;
    renderTraces();
}

const nextPage = () => {
    if (index + PER_PAGE <= traces.length) {
        index += PER_PAGE;
        renderTraces();
    }
}

const prevPage = () => {
    if (index - PER_PAGE >= 0) {
        index -= PER_PAGE;
        renderTraces();
    }
}

const Traces = () => {
    const filtered = traces.filter(filterTraces);
    const firstPage = index === 0;
    const noMore = filtered.length < (index + PER_PAGE);
    const slice = filtered.slice(index, index + PER_PAGE);

    const list = (
        <Fragment>
            {slice.map(({ timestamp, level, target, fields, span, spans }) => {
                const dt = (new Date(timestamp)).toLocaleDateString(undefined, DT_OPTIONS);
                const formattedTarget = formatTarget(target);
                const formattedSpan = formatSpan(span);
                const formatttedFields = formatFields(fields);
                return (
                    <div className="log">
                        <div className={"dt " + level}>{dt}</div>
                        <div className="target">{formattedTarget}</div>
                        {formattedSpan && (<div className="span">{formattedSpan}</div>)}
                        <div className="fields">{formatttedFields}</div>
                    </div>
                )
            })}
        </Fragment>
    )
    return (
        <div className="logs">
            <div className="filters">
                <input className="target" placeholder="target" type="text" onInput={(e) => setTargetFilter(e.currentTarget.value)} />
                <button className="level TRACE" onClick={() => setLevelFilter("TRACE")}>TRACE</button>
                <button className="level DEBUG" onClick={() => setLevelFilter("DEBUG")}>DEBUG</button>
                <button className="level INFO" onClick={() => setLevelFilter("INFO")}>INFO</button>
                <button className="level WARN" onClick={() => setLevelFilter("WARN")}>WARN</button>
                <button className="level ERROR" onClick={() => setLevelFilter("ERROR")}>ERROR</button>
            </div>

            {slice.length === 0 ? (
                <div>Nothing found</div>
            ) : (
                <Fragment>
                    <div className="pagination">
                        <button onClick={prevPage} disabled={firstPage}>{"< Newer"}</button> 
                        <button onClick={nextPage} disabled={noMore}>{"Older >"}</button> 
                    </div>
                
                    <div className="log">
                        <div className={"dt"}>timestamp</div>
                        <div className="target">target</div>
                        <div className="span">maybe span</div>
                        <div className="fields">content</div>
                    </div>

                    {list}

                    <div className="pagination">
                        <button onClick={prevPage} disabled={firstPage}>{"< Newer"}</button> 
                        <button onClick={nextPage} disabled={noMore}>{"Older >"}</button> 
                    </div>
                </Fragment>
            )}
            
        </div>        
    )
};

const filterTraces = (trace: Trace) => {
    const level = levelFilter;
    const target = targetFilter;
    if (filterByLevel(trace, level)) return false;
    if (target && !trace.target.includes(target)) return false;
    return true;
}

const filterByLevel = (trace: Trace, base: Level) => {
    const level = trace.level;
    if (base === "TRACE") return false;
    if (base === "DEBUG" && level !== "TRACE") return false;
    if (base === "INFO" && level !== "TRACE" && level !== "DEBUG") return false;
    if (base === "WARN" && level !== "TRACE" && level !== "DEBUG" && level !== "INFO") return false;
    if (base === "ERROR" && level !== "TRACE" && level !== "DEBUG" && level !== "INFO" && level !== "WARN") return false;
    return true;
}

const formatFields = (fields: Attrs) => {
    if (Object.keys(fields).length === 1 && fields.message) {
        return fields.message
    }
    if (Object.keys(fields).length === 2 && fields.job && (fields.start || fields.end)) {
        if (fields.start) return `STARTED ${fields.job}`
        else return `FINISHED ${fields.job}`
    }

    if (Object.keys(fields).length === 2 && fields.latency && fields.code) {
        return `${fields.code} in ${fields.latency} ms`
    }
    return <div className="raw">{JSON.stringify(fields)}</div>
}

const formatSpan = (span?: Attrs) => {
    if (!span) return null;
    if (Object.keys(span).length === 0) return null;
    if (Object.keys(span).length === 2 && span.job && span.name) {
        return `${span.name} "${span.job}"`
    }
    if (span?.name === "http" && span?.method && span?.uri) {
        return `-> ${span.method} ${span.uri}`
    }
    return <div className="raw">{JSON.stringify(span)}</div>
}

const formatTarget = (target: string) => {
    return target.replaceAll("::", "\n")
}