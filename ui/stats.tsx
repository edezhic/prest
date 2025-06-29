import Chart, { ChartItem } from 'chart.js/auto';

type MonitoringInfo = {
    records: Record[],
    max_ram: number,
    cores_num: number,
}

type Record = {
    timestamp: string,
    app_cpu: number,
    other_cpu: number,
    app_ram: number,
    other_ram: number,
}

let info: MonitoringInfo = {
    records: [],
    max_ram: 0,
    cores_num: 0,
};

let currentPeriod = '15m';

/**
 * Fills time gaps in system monitoring data with zero values to visually show when the app wasn't running.
 * This prevents misleading line connections across downtime periods on the chart.
 * 
 * @param records Array of monitoring records from the backend
 * @returns Array with gap periods filled with zero values
 */
const fillTimeGaps = (records: Record[]): Record[] => {
    if (records.length === 0) return records;
    
    // Sort records by timestamp to ensure proper order
    records.sort((a, b) => new Date(a.timestamp + 'Z').getTime() - new Date(b.timestamp + 'Z').getTime());
    
    const filledRecords: Record[] = [];
    const intervalMs = 1000; // 1 second sampling interval
    const gapThreshold = intervalMs * 3; // Consider gaps larger than 3 seconds as downtime
    const maxGapToFill = 5 * 60 * 1000; // Don't fill gaps longer than 5 minutes (for performance)
    
    for (let i = 0; i < records.length; i++) {
        const currentRecord = records[i];
        filledRecords.push(currentRecord);
        
        // Check if there's a next record to compare with
        if (i < records.length - 1) {
            const nextRecord = records[i + 1];
            const currentTime = new Date(currentRecord.timestamp + 'Z').getTime();
            const nextTime = new Date(nextRecord.timestamp + 'Z').getTime();
            const timeDiff = nextTime - currentTime;
            
            // Fill gaps that indicate downtime periods
            if (timeDiff > gapThreshold && timeDiff <= maxGapToFill) {
                const gapStart = currentTime + intervalMs;
                const gapEnd = nextTime - intervalMs;
                
                // Add zero records for the gap period every second
                for (let gapTime = gapStart; gapTime <= gapEnd; gapTime += intervalMs) {
                    const gapDate = new Date(gapTime);
                    // Format timestamp to match backend format (UTC without Z suffix)
                    const utcTimestamp = gapDate.toISOString().slice(0, -1);
                    const gapRecord: Record = {
                        timestamp: utcTimestamp,
                        app_cpu: 0,    // App wasn't running - zero CPU usage
                        other_cpu: 0,  // System also likely idle during app downtime
                        app_ram: 0,    // App wasn't running - zero RAM usage
                        other_ram: 0,  // System RAM also zero during downtime
                    };
                    filledRecords.push(gapRecord);
                }
            }
        }
    }
    
    return filledRecords;
};

export async function loadStats(period: string = '15m') {
    currentPeriod = period;
    const resp = await fetch(`/admin/monitoring/data?period=${period}`);
    info = await resp.json();
    info.records = fillTimeGaps(info.records);
    info.records = info.records.map(formatRecord);
    renderChart()
}

const formatPeriodLabel = (period: string): string => {
    const labels: { [key: string]: string } = {
        '5m': 'Last 5 minutes',
        '15m': 'Last 15 minutes',
        '30m': 'Last 30 minutes',
        '1h': 'Last 1 hour',
        '2h': 'Last 2 hours',
        '6h': 'Last 6 hours',
        '12h': 'Last 12 hours',
        '24h': 'Last 24 hours'
    };
    return labels[period] || 'Last 15 minutes';
};

const renderChart = () => {
    // Clear previous chart if it exists
    const canvas = document.getElementById('stats-chart') as HTMLCanvasElement;
    if (canvas && (canvas as any).chart) {
        (canvas as any).chart.destroy();
    }
    
    const chart = new Chart(
        canvas as ChartItem,
        {
            type: 'line',
            data: {
                labels: info.records.map(r => r.timestamp),
                datasets: [{
                    label: 'Total CPU',
                    data: info.records.map(r => r.app_cpu + r.other_cpu),
                    pointStyle: false,
                    borderColor: '#00598a'
                }, {
                    label: 'System RAM',
                    data: info.records.map(r => r.other_ram / info.max_ram * 100),
                    backgroundColor: '#44403c',
                    type: 'bar',
                }, {
                    label: 'App RAM',
                    data: info.records.map(r => r.app_ram / info.max_ram * 100),
                    backgroundColor: '#22c55e',
                    type: 'bar',
                },]
            },
            options: {
                interaction: {
                    intersect: false,
                    mode: 'index',
                },
                scales: {
                    x: {
                        stacked: true,
                    },
                    y: {
                        min: 0,
                        max: 100,
                        stacked: true,
                    }
                },
                plugins: {
                    tooltip: {
                        enabled: false,
                        position: 'nearest',
                        external: externalTooltipHandler
                    },
                    title: {
                        display: true,
                        text: `${info.cores_num} core(s), ${(info.max_ram / 1000).toFixed(1)} GB RAM - ${formatPeriodLabel(currentPeriod)}`,
                    }
                },
                maintainAspectRatio: false,
            }
        }
    );
    
    // Store chart reference for cleanup
    (canvas as any).chart = chart;
}

const getOrCreateTooltip = (chart) => {
    let tooltipEl = chart.canvas.parentNode.querySelector('div');

    if (!tooltipEl) {
        tooltipEl = document.createElement('div');
        tooltipEl.style.background = 'rgba(0, 0, 0, 0.7)';
        tooltipEl.style.borderRadius = '3px';
        tooltipEl.style.color = 'white';
        tooltipEl.style.opacity = 1;
        tooltipEl.style.pointerEvents = 'none';
        tooltipEl.style.position = 'absolute';
        tooltipEl.style.transform = 'translate(-50%, 0)';
        tooltipEl.style.transition = 'all .1s ease';

        const table = document.createElement('table');
        table.style.margin = '0px';

        tooltipEl.appendChild(table);
        chart.canvas.parentNode.appendChild(tooltipEl);
    }

    return tooltipEl;
};

const externalTooltipHandler = (context) => {
    // Tooltip Element
    const { chart, tooltip } = context;
    const tooltipEl = getOrCreateTooltip(chart);

    // Hide if no tooltip
    if (tooltip.opacity === 0) {
        tooltipEl.style.opacity = 0;
        return;
    }

    // Set Text
    if (tooltip.body) {
        const titleLines = tooltip.title || [];
        const bodyLines = tooltip.body.map(b => b.lines);

        const tableHead = document.createElement('thead');

        titleLines.forEach(title => {
            const tr = document.createElement('tr');
            tr.style.borderWidth = '0';

            const th = document.createElement('th');
            th.style.borderWidth = '0';
            const text = document.createTextNode(title);

            th.appendChild(text);
            tr.appendChild(th);
            tableHead.appendChild(tr);
        });

        const tableBody = document.createElement('tbody');
        bodyLines.forEach((body, i) => {
            const colors = tooltip.labelColors[i];

            const span = document.createElement('span');
            span.style.background = colors.backgroundColor;
            span.style.borderColor = colors.borderColor;
            span.style.borderWidth = '2px';
            span.style.marginRight = '10px';
            span.style.height = '10px';
            span.style.width = '10px';
            span.style.display = 'inline-block';

            const tr = document.createElement('tr');
            tr.style.backgroundColor = 'inherit';
            tr.style.borderWidth = '0';

            const td = document.createElement('td');
            td.style.borderWidth = '0';


            let [type, value] = (body.toString()).split(': ');
            value = Number(value.replace(',', '.'));
            if (type === 'System RAM') {
                type = 'System RAM usage';
                value = Math.trunc(value * info.max_ram / 100) + ' MBs';
            } else if (type === 'App RAM') {
                type = 'App RAM usage';
                value = Math.trunc(value * info.max_ram / 100) + ' MBs';
            } else if (type === 'CPU') {
                type = 'Total CPU usage';
                value += ' %';
            }
            const content = `${type}: ${value}`;

            const text = document.createTextNode(content);

            td.appendChild(span);
            td.appendChild(text);
            tr.appendChild(td);
            tableBody.appendChild(tr);
        });

        const tableRoot = tooltipEl.querySelector('table');

        // Remove old children
        while (tableRoot.firstChild) {
            tableRoot.firstChild.remove();
        }

        // Add new children
        tableRoot.appendChild(tableHead);
        tableRoot.appendChild(tableBody);
    }

    const { offsetLeft: positionX, offsetTop: positionY } = chart.canvas;

    // Display, position, and set styles for font
    tooltipEl.style.opacity = 1;
    tooltipEl.style.left = positionX + tooltip.caretX + 'px';
    tooltipEl.style.top = positionY + tooltip.caretY + 'px';
    tooltipEl.style.font = tooltip.options.bodyFont.string;
    tooltipEl.style.padding = tooltip.options.padding + 'px ' + tooltip.options.padding + 'px';
};

const formatRecord = (r: Record) => {
    return {
        ...r,
        timestamp: (new Date(r.timestamp + 'Z')).toLocaleTimeString()
    }
}