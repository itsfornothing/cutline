import React, { useState, useEffect } from 'react';
import { AlertCircle, CheckCircle2, Code2, Copy, Check } from 'lucide-react';
import Editor, { Monaco, BeforeMount } from '@monaco-editor/react';
import timelineSchema from '../../schema/timeline.json';

interface JsonEditorTabProps {
  timeline: any;
  onChange: (updatedTimeline: any) => void;
}

export const JsonEditorTab: React.FC<JsonEditorTabProps> = ({ timeline, onChange }) => {
  const [jsonString, setJsonString] = useState<string>('');
  const [parseError, setParseError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    try {
      const formatted = JSON.stringify(timeline, null, 2);
      setJsonString(formatted);
      setParseError(null);
    } catch (err: any) {
      setParseError(err.message);
    }
  }, [timeline]);

  const handleBeforeMount: BeforeMount = (monaco: Monaco) => {
    // Configure JSON diagnostics with our official Cutline Timeline schema
    monaco.languages.json.jsonDefaults.setDiagnosticsOptions({
      validate: true,
      allowComments: false,
      schemas: [
        {
          uri: 'https://cutline.dev/schema/v1.0/timeline.json',
          fileMatch: ['*'],
          schema: timelineSchema,
        },
      ],
      enableSchemaRequest: false,
    });
  };

  const handleEditorChange = (val: string | undefined) => {
    const content = val ?? '';
    setJsonString(content);
    try {
      const parsed = JSON.parse(content);
      setParseError(null);
      onChange(parsed);
    } catch (err: any) {
      setParseError(err.message);
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(jsonString);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex flex-col h-full text-xs font-mono select-none">
      {/* Editor Header Bar */}
      <div className="h-8 border-b border-white/8 px-3 flex items-center justify-between bg-panel-dark/60">
        <div className="flex items-center gap-2">
          <Code2 className="w-3.5 h-3.5 text-neutral-400" />
          <span className="text-[11px] text-neutral-400">cutline_schema.json</span>
        </div>

        <div className="flex items-center gap-2">
          {parseError ? (
            <span className="flex items-center gap-1 text-[10px] text-red-400">
              <AlertCircle className="w-3 h-3" />
              <span>SCHEMA / SYNTAX ERROR</span>
            </span>
          ) : (
            <span className="flex items-center gap-1 text-[10px] text-emerald-400">
              <CheckCircle2 className="w-3 h-3" />
              <span>VALID TIMELINE JSON</span>
            </span>
          )}

          <button
            onClick={handleCopy}
            className="p-1 text-neutral-400 hover:text-neutral-200 transition-colors"
            title="Copy JSON"
            aria-label="Copy JSON"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
          </button>
        </div>
      </div>

      {/* Real Monaco Editor Container */}
      <div className="flex-1 relative bg-canvas-dark overflow-hidden">
        <Editor
          height="100%"
          defaultLanguage="json"
          theme="vs-dark"
          value={jsonString}
          beforeMount={handleBeforeMount}
          onChange={handleEditorChange}
          options={{
            minimap: { enabled: false },
            fontSize: 12,
            lineNumbers: 'on',
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 2,
            wordWrap: 'on',
            formatOnPaste: true,
            formatOnType: true,
            suggest: {
              snippetsPreventQuickSuggestions: false,
            },
            quickSuggestions: {
              other: true,
              comments: false,
              strings: true,
            },
          }}
          loading={
            <div className="flex items-center justify-center h-full text-neutral-500 font-sans text-xs">
              Loading Monaco Editor...
            </div>
          }
        />
      </div>

      {/* Parse Error Notification */}
      {parseError && (
        <div className="p-2.5 bg-red-950/40 border-t border-red-500/20 text-red-400 text-[11px] font-mono flex items-start gap-2">
          <AlertCircle className="w-3.5 h-3.5 shrink-0 mt-0.5" />
          <span className="truncate">{parseError}</span>
        </div>
      )}
    </div>
  );
};
