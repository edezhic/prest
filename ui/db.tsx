/**  @jsx h @jsxFrag Fragment */
import { h, render, Fragment } from "preact";

type FieldSchema = {
    name: string;
    sql_type: string;
    rust_type: string;
    pkey: boolean;
    numeric: boolean;
    list: boolean;
    optional: boolean;
    serialized: boolean;
};

type TableDescription = {
    name: string;
    fields: FieldSchema[];
};

type TableData = {
    name: string;
    fields: FieldSchema[];
    rows: string[][];
    has_more: boolean;
    total_pages?: number;
};

type EditingRow = {
    tableIndex: number;
    rowIndex: number;
    values: { [key: string]: string };
} | null;

// Initialize state variables with proper defaults
let schema: TableDescription[] = [];
let tableData: { [tableName: string]: TableData } = {};
let selectedTable: string | null = null;
let editingRow: EditingRow = null;
let isCreating: boolean = false;
let newRowValues: { [key: string]: string } = {};
let currentPage: number = 0;
const PAGE_SIZE = 20;

// Ensure schema is always an array
function getSchema(): TableDescription[] {
    return Array.isArray(schema) ? schema : [];
}

async function loadSchema() {
    try {
        const resp = await fetch(`/admin/db/schema`);
        const data = await resp.json();
        schema = data || [];
        renderDatabase();
    } catch (error) {
        console.error('Failed to load schema:', error);
        schema = []; // Ensure schema is always initialized
        renderDatabase();
    }
}

async function loadTableData(tableName: string, page: number = 0) {
    const offset = page * PAGE_SIZE;
    
    try {
        const resp = await fetch(`/admin/db/table/${tableName}?offset=${offset}&limit=${PAGE_SIZE}`);
        const data = await resp.json();
        tableData[tableName] = data;
        currentPage = page;
        renderDatabase();
    } catch (error) {
        console.error('Failed to load table data:', error);
    }
}

function selectTable(tableName: string) {
    selectedTable = tableName;
    currentPage = 0;
    // Clear cached data to force reload with pagination
    delete tableData[tableName];
    loadTableData(tableName, 0);
}

function nextDbPage() {
    if (!selectedTable) return;
    const table = tableData[selectedTable];
    if (table && table.has_more) {
        loadTableData(selectedTable, currentPage + 1);
    }
}

function prevDbPage() {
    if (!selectedTable || currentPage === 0) return;
    loadTableData(selectedTable, currentPage - 1);
}

function startEditing(tableIndex: number, rowIndex: number) {
    const table = tableData[selectedTable!];
    const row = table.rows[rowIndex];
    const values: { [key: string]: string } = {};
    
    table.fields.forEach((field, index) => {
        let value = row[index] || '';
        // For boolean fields, ensure we have proper true/false values
        if (field.sql_type === 'Boolean' && !field.list && !field.optional) {
            value = value === 'true' ? 'true' : 'false';
        }
        // For JSON fields, format for display in textarea
        else if (field.serialized && value) {
            try {
                const parsed = JSON.parse(value);
                value = JSON.stringify(parsed, null, 2);
            } catch (e) {
                // If it's not valid JSON, keep as string
                console.warn('Invalid JSON in field', field.name, ':', value);
            }
        }
        values[field.name] = value;
    });
    
    editingRow = { tableIndex, rowIndex, values };
    renderDatabase();
}

function updateEditValue(fieldName: string, value: string) {
    if (editingRow) {
        editingRow.values[fieldName] = value;
    } else if (isCreating) {
        newRowValues[fieldName] = value;
    }
}

function cancelEdit() {
    editingRow = null;
    isCreating = false;
    newRowValues = {};
    renderDatabase();
}

async function saveEdit() {
    if (!selectedTable || !editingRow) return;
    
    const table = tableData[selectedTable];
    
    // Process special fields - convert strings to proper types
    const processedValues: { [key: string]: any } = { ...editingRow.values };
    table.fields.forEach(field => {
        if (field.serialized && processedValues[field.name]) {
            try {
                processedValues[field.name] = JSON.parse(processedValues[field.name]);
            } catch (e) {
                console.error('Invalid JSON in field', field.name, ':', processedValues[field.name]);
                return; // Don't submit if JSON is invalid
            }
        }
        // Convert boolean string values to actual booleans
        else if (field.sql_type === 'Boolean' && !field.list && !field.optional) {
            processedValues[field.name] = processedValues[field.name] === 'true';
        }
    });
    
    try {
        const method = 'PATCH';
        const resp = await fetch(`/admin/db/table/${selectedTable}`, {
            method,
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(processedValues)
        });
        
        if (resp.ok) {
            // Clear edit state first
            editingRow = null;
            // Reload table data at current page
            delete tableData[selectedTable];
            await loadTableData(selectedTable, currentPage);
        }
    } catch (error) {
        console.error('Failed to save edit:', error);
    }
}

async function createRow() {
    if (!selectedTable) return;
    
    const table = tableData[selectedTable];
    
    // Process special fields - convert strings to proper types  
    const processedValues: { [key: string]: any } = { ...newRowValues };
    table.fields.forEach(field => {
        if (field.serialized && processedValues[field.name]) {
            try {
                processedValues[field.name] = JSON.parse(processedValues[field.name]);
            } catch (e) {
                console.error('Invalid JSON in field', field.name, ':', processedValues[field.name]);
                return; // Don't submit if JSON is invalid
            }
        }
        // Convert boolean string values to actual booleans
        else if (field.sql_type === 'Boolean' && !field.list && !field.optional) {
            processedValues[field.name] = processedValues[field.name] === 'true';
        }
    });
    
    try {
        const resp = await fetch(`/admin/db/table/${selectedTable}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(processedValues)
        });
        
        if (resp.ok) {
            // Clear creation state first
            isCreating = false;
            newRowValues = {};
            // Reload table data at current page
            delete tableData[selectedTable];
            await loadTableData(selectedTable, currentPage);
        }
    } catch (error) {
        console.error('Failed to create row:', error);
    }
}

async function deleteRow(rowIndex: number) {
    if (!selectedTable || !confirm('Are you sure you want to delete this row?')) return;
    
    const table = tableData[selectedTable];
    const row = table.rows[rowIndex];
    
    // Create object with all field values for deletion
    const rowData: { [key: string]: any } = {};
    table.fields.forEach((field, index) => {
        let value: any = row[index] || '';
        // Convert boolean string values to actual booleans
        if (field.sql_type === 'Boolean' && !field.list && !field.optional) {
            value = value === 'true';
        }
        // Parse JSON fields
        else if (field.serialized && value) {
            try {
                value = JSON.parse(value);
            } catch (e) {
                // Keep as string if parsing fails
                console.warn('Could not parse JSON for deletion:', value);
            }
        }
        rowData[field.name] = value;
    });
    
    try {
        const resp = await fetch(`/admin/db/table/${selectedTable}`, {
            method: 'DELETE',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(rowData)
        });
        
        if (resp.ok) {
            // Reload table data at current page
            delete tableData[selectedTable];
            await loadTableData(selectedTable, currentPage);
        }
    } catch (error) {
        console.error('Failed to delete row:', error);
    }
}

function startCreating() {
    if (!selectedTable) return;
    
    isCreating = true;
    newRowValues = {};
    
    // Initialize with default values
    const table = tableData[selectedTable];
    table.fields.forEach(field => {
        // For boolean fields, default to false
        if (field.sql_type === 'Boolean' && !field.list && !field.optional) {
            newRowValues[field.name] = 'false';
        }
        // For JSON fields, default to empty object
        else if (field.serialized) {
            newRowValues[field.name] = '{}';
        } else {
            newRowValues[field.name] = '';
        }
    });
    
    renderDatabase();
}

function getInputType(field: FieldSchema): string {
    if (field.serialized) {
        return 'json';
    } else if (field.sql_type === 'Boolean' && !field.list && !field.optional) {
        return 'checkbox';
    } else if (field.numeric && !field.list && !field.optional) {
        return 'number';
    }
    return 'text';
}

function formatFieldType(field: FieldSchema): string {
    return `${field.name} (${field.rust_type})`;
}

const DatabaseAdmin = () => {
    const currentSchema = getSchema();
    
    if (currentSchema.length === 0) {
        return (
            <div className="db-admin">
                <div>Loading schema...</div>
            </div>
        );
    }

    const renderTableList = () => (
        <Fragment>
            <h2>Database Tables</h2>
            <div className="table-list">
                {currentSchema.map(table => (
                    <button 
                        key={table.name}
                        className={`table-button ${selectedTable === table.name ? 'selected' : ''}`}
                        onClick={() => selectTable(table.name)}
                    >
                        {table.name} ({table.fields.length} fields)
                    </button>
                ))}
            </div>
        </Fragment>
    );

    const renderTable = () => {
        if (!selectedTable) {
            return <div className="table-placeholder">Please select a table from above to view its data</div>;
        }
        
        if (!tableData[selectedTable]) {
            return <div className="table-loading">Loading table data...</div>;
        }

        const table = tableData[selectedTable];
        
        return (
            <div className="table-view">
                <div className="table-header">
                    <h2>{table.name}</h2>
                    <div className="table-controls">
                        <div className="pagination-info">
                            Page {currentPage + 1} ({table.rows.length} rows)
                        </div>
                        <div className="pagination-controls">
                            <button onClick={prevDbPage} disabled={currentPage === 0}>
                                ← Previous
                            </button>
                            <button onClick={nextDbPage} disabled={!table.has_more}>
                                Next →
                            </button>
                        </div>
                        <button onClick={startCreating} disabled={isCreating}>
                            Add New Row
                        </button>
                    </div>
                </div>
                
                <table className="data-table">
                    <thead>
                        <tr>
                            {table.fields.map(field => (
                                <th key={field.name}>
                                    {formatFieldType(field)}
                                    {field.pkey && <span className="primary-key">🔑</span>}
                                </th>
                            ))}
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {isCreating && (
                            <tr className="creating-row">
                                {table.fields.map(field => (
                                    <td key={field.name}>
                                        {field.serialized ? (
                                            <textarea
                                                value={newRowValues[field.name] || '{}'}
                                                onInput={(e) => updateEditValue(field.name, e.currentTarget.value)}
                                                rows={3}
                                                className="json-field"
                                                placeholder={field.pkey ? "Enter primary key as JSON..." : ""}
                                            />
                                        ) : (
                                            <input
                                                type={getInputType(field)}
                                                value={getInputType(field) === 'checkbox' ? undefined : (newRowValues[field.name] || '')}
                                                checked={getInputType(field) === 'checkbox' ? newRowValues[field.name] === 'true' : undefined}
                                                onInput={(e) => {
                                                    const isCheckbox = getInputType(field) === 'checkbox';
                                                    if (isCheckbox) {
                                                        updateEditValue(field.name, e.currentTarget.checked ? "true" : "false");
                                                    } else {
                                                        updateEditValue(field.name, e.currentTarget.value);
                                                    }
                                                }}
                                                placeholder={field.pkey ? "Enter primary key..." : ""}
                                            />
                                        )}
                                    </td>
                                ))}
                                <td>
                                    <button onClick={createRow}>✓</button>
                                    <button onClick={cancelEdit}>✗</button>
                                </td>
                            </tr>
                        )}
                        
                        {table.rows.map((row, rowIndex) => {
                            const isEditing = editingRow?.rowIndex === rowIndex;
                            
                            return (
                                <tr key={rowIndex} className={isEditing ? 'editing-row' : 'view-row'}>
                                    {table.fields.map((field, fieldIndex) => (
                                        <td key={field.name}>
                                            {isEditing ? (
                                                field.serialized ? (
                                                    <textarea
                                                        value={editingRow?.values[field.name] || '{}'}
                                                        onInput={(e) => updateEditValue(field.name, e.currentTarget.value)}
                                                        disabled={field.pkey}
                                                        rows={3}
                                                        className="json-field"
                                                    />
                                                ) : (
                                                    <input
                                                        type={getInputType(field)}
                                                        value={getInputType(field) === 'checkbox' ? undefined : (editingRow?.values[field.name] || '')}
                                                        checked={getInputType(field) === 'checkbox' ? editingRow?.values[field.name] === 'true' : undefined}
                                                                                                        onInput={(e) => {
                                                    const isCheckbox = getInputType(field) === 'checkbox';
                                                    if (isCheckbox) {
                                                        updateEditValue(field.name, e.currentTarget.checked ? "true" : "false");
                                                    } else {
                                                        updateEditValue(field.name, e.currentTarget.value);
                                                    }
                                                }}
                                                        disabled={field.pkey}
                                                    />
                                                )
                                            ) : (
                                                <span>{row[fieldIndex]}</span>
                                            )}
                                        </td>
                                    ))}
                                    <td>
                                                                                {isEditing ? (
                                            <Fragment>
                                                <button onClick={saveEdit}>✓</button>
                                                <button onClick={cancelEdit}>✗</button>
                                            </Fragment>
                                        ) : (
                            <Fragment>
                                <button onClick={() => startEditing(0, rowIndex)}>✏️</button>
                                <button onClick={() => deleteRow(rowIndex)}>🗑️</button>
                            </Fragment>
                        )}
                                    </td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>
        );
    };

    return (
        <div className="db-admin">
            <div className="db-layout">
                <div className="table-navigation">
                    {renderTableList()}
                </div>
                <div className="table-content">
                    {renderTable()}
                </div>
            </div>
        </div>
    );
};

// Render function
function renderDatabase() {
    const container = document.getElementById('db-container');
    if (container) {
        render(<DatabaseAdmin />, container);
    }
}

// Make loadSchema globally available for HTMX trigger
if (typeof window !== 'undefined') {
    (window as any).loadSchema = loadSchema;
}
