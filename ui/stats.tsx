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

export async function loadStats(period: string) {
    const resp = await fetch(`/admin/monitoring/data`);
    info = await resp.json();
    info.records = info.records.map(formatRecord);
    renderChart()
}

const renderChart = () => {
    new Chart(
        document.getElementById('stats-chart') as ChartItem,
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
                        text: `${info.cores_num} core(s), ${(info.max_ram / 1000).toFixed(1)} GB RAM`,
                    }
                },
                maintainAspectRatio: false,
            }
        }
    );
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