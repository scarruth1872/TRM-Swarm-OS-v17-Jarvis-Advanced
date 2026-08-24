import React, { useState, useEffect } from "react";
import { 
  Folder, 
  Mail, 
  Calendar as CalendarIcon, 
  FileText, 
  Video, 
  Users, 
  Search, 
  Plus, 
  Check, 
  Trash2, 
  AlertCircle, 
  ExternalLink, 
  Lock, 
  RefreshCw, 
  FileSpreadsheet, 
  Presentation, 
  StickyNote, 
  Clock, 
  Send,
  UserPlus,
  Compass,
  FileCode,
  Sparkles,
  ArrowRight,
  User,
  Power,
  ShieldCheck,
  CheckCircle,
  Copy,
  Github
} from "lucide-react";
import { doc, getDoc, setDoc, updateDoc, arrayUnion, deleteDoc, collection, getDocs, addDoc } from "firebase/firestore";
import { auth, db, OperationType, handleFirestoreError } from "../lib/firebase";

interface WorkspaceNexusProps {
  accessToken: string | null;
  onAuthorize: () => void;
  speakText?: (text: string) => void;
  authError?: string | null;
}

// Local Interfaces
interface DriveFile {
  id: string;
  name: string;
  mimeType: string;
  modifiedTime: string;
  webViewLink?: string;
  owners?: Array<{ displayName: string; emailAddress: string; photoLink?: string }>;
}

interface GmailMessage {
  id: string;
  threadId: string;
  subject: string;
  from: string;
  date: string;
  snippet: string;
  body?: string;
}

interface CalendarEvent {
  id: string;
  summary: string;
  description?: string;
  start: { dateTime?: string; date?: string };
  end: { dateTime?: string; date?: string };
  htmlLink?: string;
  hangoutLink?: string;
}

interface KeepNote {
  id: string;
  title: string;
  content: string;
  color: string;
  createdAt: string;
  tags: string[];
}

interface PeopleContact {
  id: string;
  name: string;
  email?: string;
  phone?: string;
  resourceName: string;
}

export const WorkspaceNexus: React.FC<WorkspaceNexusProps> = ({ accessToken, onAuthorize, speakText, authError }) => {
  // Service Tabs
  const [activeService, setActiveService] = useState<"drive" | "gmail" | "calendar" | "docs_sheets" | "slides_meet" | "keep" | "contacts" | "github">("drive");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  // Google Drive Ingestion Learning state
  const [learningDriveId, setLearningDriveId] = useState<string | null>(null);

  // GitHub integration states
  const [githubRepo, setGithubRepo] = useState("shawncarruth/jarvis-os-moe-hub");
  const [githubToken, setGithubToken] = useState("");
  const [githubBranch, setGithubBranch] = useState("main");
  const [githubPath, setGithubPath] = useState("");
  const [githubFiles, setGithubFiles] = useState<any[]>([]);
  const [loadingGithub, setLoadingGithub] = useState(false);
  const [learningGithubPath, setLearningGithubPath] = useState<string | null>(null);

  // General Ingestion Learning result display
  const [learningStatus, setLearningStatus] = useState<"idle" | "learning" | "success" | "error">("idle");
  const [learnSummary, setLearnSummary] = useState<string | null>(null);
  const [learnDetails, setLearnDetails] = useState<any>(null);

  // Search filter
  const [searchQuery, setSearchQuery] = useState("");

  // Data Collections
  const [driveFiles, setDriveFiles] = useState<DriveFile[]>([]);
  const [emails, setEmails] = useState<GmailMessage[]>([]);
  const [calendarEvents, setCalendarEvents] = useState<CalendarEvent[]>([]);
  const [keepNotes, setKeepNotes] = useState<KeepNote[]>([]);
  const [contacts, setContacts] = useState<PeopleContact[]>([]);

  // Selected details
  const [selectedEmail, setSelectedEmail] = useState<GmailMessage | null>(null);

  // Forms / States
  const [showConfirmModal, setShowConfirmModal] = useState(false);
  const [pendingAction, setPendingAction] = useState<{ type: string; trigger: () => Promise<void>; text: string } | null>(null);

  // Google Drive upload simple state
  const [newFileName, setNewFileName] = useState("");
  const [newFileContent, setNewFileContent] = useState("");

  // Gmail compose simple state
  const [mailTo, setMailTo] = useState("");
  const [mailSubject, setMailSubject] = useState("");
  const [mailBody, setMailBody] = useState("");

  // Calendar create state
  const [calSummary, setCalSummary] = useState("");
  const [calStart, setCalStart] = useState("");
  const [calEnd, setCalEnd] = useState("");
  const [calDesc, setCalDesc] = useState("");
  const [calIncludeMeet, setCalIncludeMeet] = useState(true);

  // Sheets & Docs creation states
  const [sheetTitle, setSheetTitle] = useState("");
  const [docTitle, setDocTitle] = useState("");
  const [docContent, setDocContent] = useState("");

  // Slides & Meet states
  const [slidesTitle, setSlidesTitle] = useState("");
  const [meetConferenceCode, setMeetConferenceCode] = useState("");
  const [createdMeetUrl, setCreatedMeetUrl] = useState<string | null>(null);

  // NotebookLM study-guide slide designer states
  const [nbLMTopic, setNbLMTopic] = useState("");
  const [nbLMTheme, setNbLMTheme] = useState("notebooklm_ivory");
  const [selectedDriveFileIds, setSelectedDriveFileIds] = useState<string[]>([]);
  const [selectedNoteIds, setSelectedNoteIds] = useState<string[]>([]);
  const [nbLMPending, setNbLMPending] = useState(false);

  // Keep Notes states
  const [noteTitle, setNoteTitle] = useState("");
  const [noteContent, setNoteContent] = useState("");
  const [noteColor, setNoteColor] = useState("charcoal"); // default theme
  const [noteTags, setNoteTags] = useState("");
  const [keepSyncStatus, setKeepSyncStatus] = useState<"synced" | "local" | "error">("local");
  const [syncStatusDetail, setSyncStatusDetail] = useState<string | null>(null);

  // Contact create state
  const [contactGivenName, setContactGivenName] = useState("");
  const [contactFamilyName, setContactFamilyName] = useState("");
  const [contactEmail, setContactEmail] = useState("");
  const [contactPhone, setContactPhone] = useState("");

  // Trigger speech assistance safely
  const triggerVoiceUpdate = (text: string) => {
    if (speakText) {
      speakText(text);
    }
  };

  // Run on Tab Change
  useEffect(() => {
    if (activeService !== "github" && !accessToken) return;
    setSearchQuery("");
    setError(null);
    setSuccessMessage(null);
    setSelectedEmail(null);

    switch (activeService) {
      case "drive":
        fetchDriveFiles();
        break;
      case "gmail":
        fetchEmails();
        break;
      case "calendar":
        fetchCalendarEvents();
        break;
      case "docs_sheets":
        fetchDriveFiles(); // Sheets and Doc search are Drive operations
        break;
      case "slides_meet":
        fetchDriveFiles();
        break;
      case "keep":
        fetchKeepNotes();
        break;
      case "contacts":
        fetchContacts();
        break;
      case "github":
        if (githubFiles.length === 0) {
          fetchGithubRepoContents("");
        }
        break;
    }
  }, [activeService, accessToken]);

  // ------------ API INVOCATIONS ------------

  // 1. Google Drive — Fetch files
  const fetchDriveFiles = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(
        `https://www.googleapis.com/drive/v3/files?pageSize=500&fields=files(id,name,mimeType,modifiedTime,owners,webViewLink)&orderBy=modifiedTime%20desc`,
        {
          headers: { Authorization: `Bearer ${accessToken}` },
        }
      );
      if (!response.ok) throw new Error("Drive service communication aborted by host.");
      const data = await response.json();
      setDriveFiles(data.files || []);
    } catch (err: any) {
      setError(err.message || "Failed to sync assets from Google Drive.");
    } finally {
      setLoading(false);
    }
  };

  // Google Drive — Download content and Feed into central learn system
  const handleLearnDriveFile = async (file: DriveFile) => {
    setLearningDriveId(file.id);
    setError(null);
    setLearningStatus("learning");
    try {
      let contentText = `Google Drive File Name: ${file.name}\nMimeType: ${file.mimeType}\nModifiedTime: ${file.modifiedTime}`;
      if (file.owners && file.owners.length > 0) {
        contentText += `\nOwner: ${file.owners[0].displayName || "Unknown"}`;
      }

      // Try to download actual content if appropriate
      if (accessToken) {
        try {
          const isGoogleApp = file.mimeType.startsWith("application/vnd.google-apps.");
          const fetchUrl = isGoogleApp
            ? `https://www.googleapis.com/drive/v3/files/${file.id}/export?mimeType=text/plain`
            : `https://www.googleapis.com/drive/v3/files/${file.id}?alt=media`;

          const rawResponse = await fetch(fetchUrl, {
            headers: { Authorization: `Bearer ${accessToken}` }
          });
          if (rawResponse.ok) {
            const rawText = await rawResponse.text();
            if (rawText && !rawText.trim().startsWith("<html") && rawText.length > 5) {
              contentText += `\n\nFile Extracted Text Content:\n${rawText}`;
            }
          }
        } catch (innerErr) {
          console.warn("Couldn't pull direct text from doc. Using file meta context.", innerErr);
        }
      }

      const response = await fetch("/api/learn-resource", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "x-user-id": auth.currentUser?.uid || ""
        },
        body: JSON.stringify({
          resourceName: file.name,
          content: contentText,
          sourceType: "google_drive"
        })
      });

      if (!response.ok) throw new Error("Cognitive terminal rejected learning stream.");
      const result = await response.json();

      if (result.success && result.learnedStructure) {
        const learned = result.learnedStructure;
        setLearnSummary(learned.summary);
        setLearnDetails(learned);
        setLearningStatus("success");
        setSuccessMessage(`Cognitive ingestion fully integrated for Drive file '${file.name}'!`);
        triggerVoiceUpdate(`Google Drive asset ${file.name} is now compiled into my active memory graphs and synaptic thresholds.`);
      } else {
        throw new Error("Learning backend didn't return valid mapped structures.");
      }
    } catch (err: any) {
      setError(err.message || "Failed to learn from Drive asset.");
      setLearningStatus("error");
    } finally {
      setLearningDriveId(null);
    }
  };

  // GitHub — Fetch repo directories/files
  const fetchGithubRepoContents = async (path = "") => {
    setLoadingGithub(true);
    setError(null);
    try {
      const cleanRepo = githubRepo.replace(/https:\/\/github\.com\//, "").replace(/\/$/, "");
      const [owner, repo] = cleanRepo.split("/");
      if (!owner || !repo) {
        throw new Error("Repository format must be 'owner/repo' or standard GitHub URL.");
      }

      const headers: HeadersInit = {
        "Accept": "application/vnd.github.v3+json"
      };
      if (githubToken) {
        headers["Authorization"] = `token ${githubToken}`;
      }

      const refQuery = githubBranch ? `?ref=${githubBranch}` : "";
      const url = `https://api.github.com/repos/${owner}/${repo}/contents/${path}${refQuery}`;

      const response = await fetch(url, { headers });
      if (!response.ok) {
        throw new Error(`GitHub API error: ${response.status} ${response.statusText}. Ensure repo is public or valid scope token is passed.`);
      }

      const data = await response.json();
      const filesList = Array.isArray(data) ? data : [data];
      setGithubFiles(filesList);
      setGithubPath(path);
    } catch (err: any) {
      setError(err.message || "Failed to index GitHub repository structure.");
    } finally {
      setLoadingGithub(false);
    }
  };

  // GitHub — Download raw code and Map to Ternary Synapse
  const handleLearnGithubFile = async (file: any) => {
    setLearningGithubPath(file.path);
    setError(null);
    setLearningStatus("learning");
    try {
      const headers: HeadersInit = {
        "Accept": "application/vnd.github.v3.raw"
      };
      if (githubToken) {
        headers["Authorization"] = `token ${githubToken}`;
      }

      let rawCode = "";
      try {
        const response = await fetch(file.url, { headers });
        if (response.ok) {
          rawCode = await response.text();
        } else {
          throw new Error("Unable to read direct code payload stream.");
        }
      } catch (inner) {
        rawCode = `GitHub File Module details:\nPath: ${file.path}\nUrl: ${file.html_url}\nSize: ${file.size} bytes\nType: ${file.type}`;
      }

      const response = await fetch("/api/learn-resource", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "x-user-id": auth.currentUser?.uid || ""
        },
        body: JSON.stringify({
          resourceName: `${githubRepo}/${file.path}`,
          content: rawCode,
          sourceType: "github"
        })
      });

      if (!response.ok) throw new Error("Cognitive endpoint rejected integration.");
      const result = await response.json();

      if (result.success && result.learnedStructure) {
        const learned = result.learnedStructure;
        setLearnSummary(learned.summary);
        setLearnDetails(learned);
        setLearningStatus("success");
        setSuccessMessage(`Successfully learnt and constructed active code node: '${file.name}'`);
        triggerVoiceUpdate(`GitHub file ${file.name} successfully integrated into my active network matrices.`);
      } else {
        throw new Error("Analysis failed to produce structured cognitive node mappings.");
      }
    } catch (err: any) {
      setError(err.message || "Failed to ingest GitHub unit.");
      setLearningStatus("error");
    } finally {
      setLearningGithubPath(null);
    }
  };

  // Google Drive — Upload raw text file
  const handleCreateTextFile = async () => {
    if (!newFileName) {
      setError("Asset filename is required.");
      return;
    }

    const action = async () => {
      setLoading(true);
      setError(null);
      try {
        // Step A: Create metadata block
        const metadata = {
          name: newFileName.endsWith(".txt") ? newFileName : `${newFileName}.txt`,
          mimeType: "text/plain",
        };

        const form = new FormData();
        form.append("metadata", new Blob([JSON.stringify(metadata)], { type: "application/json" }));
        form.append("file", new Blob([newFileContent], { type: "text/plain" }));

        const response = await fetch(
          "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&fields=id,name,webViewLink",
          {
            method: "POST",
            headers: { Authorization: `Bearer ${accessToken}` },
            body: form,
          }
        );

        if (!response.ok) throw new Error("Drive asset upload failed at server node.");
        const file = await response.json();

        setSuccessMessage(`Document file '${file.name}' created in Drive sandbox!`);
        setNewFileName("");
        setNewFileContent("");
        fetchDriveFiles();
        triggerVoiceUpdate(`Document uploaded to drive successfully, Boss.`);
      } catch (err: any) {
        setError(err.message || "Uplink of diagnostic document failed.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation("Create Text File", action, `Upload text file '${newFileName}' directly to your Google Drive directory.`);
  };

  // 2. Gmail — Fetch emails
  const fetchEmails = async () => {
    setLoading(true);
    setError(null);
    try {
      const listResponse = await fetch(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages?maxResults=10",
        {
          headers: { Authorization: `Bearer ${accessToken}` },
        }
      );
      if (!listResponse.ok) throw new Error("Gmail indexing uplink failed.");
      const listData = await listResponse.json();

      if (!listData.messages || listData.messages.length === 0) {
        setEmails([]);
        return;
      }

      // Fetch message details in batch
      const emailDetails = await Promise.all(
        listData.messages.map(async (msg: { id: string }) => {
          const detailRes = await fetch(
            `https://gmail.googleapis.com/gmail/v1/users/me/messages/${msg.id}?format=full`,
            {
              headers: { Authorization: `Bearer ${accessToken}` },
            }
          );
          if (!detailRes.ok) return null;
          const detail = await detailRes.json();
          
          const headers = detail.payload?.headers || [];
          const subject = headers.find((h: any) => h.name.toLowerCase() === "subject")?.value || "No Subject";
          const from = headers.find((h: any) => h.name.toLowerCase() === "from")?.value || "Unknown Sender";
          const date = headers.find((h: any) => h.name.toLowerCase() === "date")?.value || "Unknown Date";

          return {
            id: detail.id,
            threadId: detail.threadId,
            subject,
            from,
            date,
            snippet: detail.snippet || "",
            body: detail.snippet
          };
        })
      );

      setEmails(emailDetails.filter(Boolean) as GmailMessage[]);
    } catch (err: any) {
      setError(err.message || "Failed to load sub-mail streams from Inbox.");
    } finally {
      setLoading(false);
    }
  };

  // Gmail — Send email
  const handleSendEmail = async () => {
    if (!mailTo || !mailSubject || !mailBody) {
      setError("All email dispatch parameters are required (Recipient, Subject, Body).");
      return;
    }

    const action = async () => {
      setLoading(true);
      setError(null);
      try {
        const emailLines = [
          `To: ${mailTo}`,
          `Subject: ${mailSubject}`,
          "Content-Type: text/html; charset=utf-8",
          "MIME-Version: 1.0",
          "",
          `<div style="font-family: sans-serif; padding: 20px; line-height: 1.6; color: #1e293b;">
            <div style="border-left: 4px solid #3b82f6; padding-left: 15px; margin-bottom: 20px;">
              <h3 style="margin: 0; color: #1e3a8a;">J.A.R.V.I.S. Neural System Transmission</h3>
              <p style="margin: 5px 0 0 0; font-size: 11px; color: #64748b; font-family: monospace;">SECURE_TUNNEL // USER_LINK: AUTHORIZED</p>
            </div>
            <p>${mailBody.replace(/\n/g, "<br/>")}</p>
            <hr style="border: none; border-top: 1px solid #e2e8f0; margin: 30px 0 15px 0;" />
            <p style="font-size: 11px; color: #94a3b8; font-style: italic;">
              This payload was dispatched securely via J.A.R.V.I.S. OS Core Workspace, configured by Host Admin Shawn Carruth.
            </p>
          </div>`
        ];

        // Safe Base64URL encoder
        const encodedEmail = btoa(unescape(encodeURIComponent(emailLines.join("\r\n"))))
          .replace(/\+/g, "-")
          .replace(/\//g, "_")
          .replace(/=+$/, "");

        const response = await fetch(
          "https://gmail.googleapis.com/gmail/v1/users/me/messages/send",
          {
            method: "POST",
            headers: {
              Authorization: `Bearer ${accessToken}`,
              "Content-Type": "application/json",
            },
            body: JSON.stringify({ raw: encodedEmail }),
          }
        );

        if (!response.ok) throw new Error("Mail submission refused by secure SMTP node.");

        setSuccessMessage(`Email payload routed to '${mailTo}' successfully!`);
        setMailTo("");
        setMailSubject("");
        setMailBody("");
        fetchEmails();
        triggerVoiceUpdate(`Payload transmission dispatched to recipient, Boss.`);
      } catch (err: any) {
        setError(err.message || "Dispatch protocol failed.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation("Send Email Transmission", action, `Uplink securely and transmit this email to recipient '${mailTo}' with subject '${mailSubject}'.`);
  };

  // 3. Google Calendar — Fetch events
  const fetchCalendarEvents = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(
        "https://www.googleapis.com/calendar/v3/calendars/primary/events?orderBy=startTime&singleEvents=true&maxResults=15&timeMin=" + new Date().toISOString(),
        {
          headers: { Authorization: `Bearer ${accessToken}` },
        }
      );
      if (!response.ok) throw new Error("Calendar service failed to push agenda datasets.");
      const data = await response.json();
      setCalendarEvents(data.items || []);
    } catch (err: any) {
      setError(err.message || "Calendar loading protocol faulted.");
    } finally {
      setLoading(false);
    }
  };

  // Google Calendar — Create event
  const handleCreateCalendarEvent = async () => {
    if (!calSummary || !calStart || !calEnd) {
      setError("Calendar summary, start time, and end time are required.");
      return;
    }

    const action = async () => {
      setLoading(true);
      setError(null);
      try {
        const conferenceData = calIncludeMeet ? {
          createRequest: {
            requestId: `meet_link_${Date.now()}`,
            conferenceSolutionKey: { type: "hangoutsMeet" }
          }
        } : undefined;

        const body: any = {
          summary: calSummary,
          description: calDesc,
          start: { dateTime: new Date(calStart).toISOString() },
          end: { dateTime: new Date(calEnd).toISOString() },
        };

        if (conferenceData) {
          body.conferenceData = conferenceData;
        }

        const response = await fetch(
          `https://www.googleapis.com/calendar/v3/calendars/primary/events?conferenceDataVersion=1`,
          {
            method: "POST",
            headers: {
              Authorization: `Bearer ${accessToken}`,
              "Content-Type": "application/json",
            },
            body: JSON.stringify(body),
          }
        );

        if (!response.ok) throw new Error("Calendar server refused event parameters.");
        const event = await response.json();

        setSuccessMessage(`Event '${event.summary}' scheduled on your calendar!`);
        if (event.hangoutLink) {
          setCreatedMeetUrl(event.hangoutLink);
        }
        setCalSummary("");
        setCalStart("");
        setCalEnd("");
        setCalDesc("");
        fetchCalendarEvents();
        triggerVoiceUpdate(`Scheduled briefing event successfully compiled, Boss.`);
      } catch (err: any) {
        setError(err.message || "Failed to schedule conference briefing.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation("Schedule Calendar Event", action, `Inject scheduling entry: '${calSummary}' starting ${new Date(calStart).toLocaleString()} and request Meet video endpoints.`);
  };

  // 4. Google Sheets — Create spreadsheet
  const handleCreateSheet = async () => {
    if (!sheetTitle) {
      setError("Spreadsheet designation title is required.");
      return;
    }

    const action = async () => {
      setLoading(true);
      setError(null);
      try {
        const response = await fetch(
          "https://sheets.googleapis.com/v4/spreadsheets",
          {
            method: "POST",
            headers: {
              Authorization: `Bearer ${accessToken}`,
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              properties: { title: sheetTitle },
              sheets: [
                {
                  properties: { title: "Telemetry Audit" },
                  data: [
                    {
                      startRow: 0,
                      startColumn: 0,
                      rowData: [
                        {
                          values: [
                            { userEnteredValue: { stringValue: "TIMESTAMP" } },
                            { userEnteredValue: { stringValue: "CORE_METRIC" } },
                            { userEnteredValue: { stringValue: "COGNITIVE_LOG" } }
                          ]
                        },
                        {
                          values: [
                            { userEnteredValue: { stringValue: new Date().toLocaleString() } },
                            { userEnteredValue: { stringValue: "SYNC_INITIALIZED" } },
                            { userEnteredValue: { stringValue: "Log matrix compiled dynamically via Jarvis Workspace Interface." } }
                          ]
                        }
                      ]
                    }
                  ]
                }
              ]
            }),
          }
        );

        if (!response.ok) throw new Error("Sheets database rejected initialization grid.");
        const data = await response.json();

        setSuccessMessage(`Spreadsheet grid '${data.properties.title}' created successfully!`);
        setSheetTitle("");
        fetchDriveFiles();
        triggerVoiceUpdate(`Database spreadsheet created, ready for analytical queries.`);
      } catch (err: any) {
        setError(err.message || "Spreadsheet generation faulted.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation("Create Neural Spreadsheet", action, `Create sheets matrix spreadsheet titled '${sheetTitle}' with automated diagnostic records.`);
  };

  // 4b. Google Docs — Create Document
  const handleCreateDoc = async () => {
    if (!docTitle) {
      setError("Document title is required.");
      return;
    }

    const action = async () => {
      setLoading(true);
      setError(null);
      try {
        // Create full formatted Google doc
        const createRes = await fetch("https://docs.googleapis.com/v1/documents", {
          method: "POST",
          headers: {
            Authorization: `Bearer ${accessToken}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ title: docTitle }),
        });

        if (!createRes.ok) throw new Error("Docs editor node rejected write instructions.");
        const docObj = await createRes.json();

        // Write content into Document immediately
        if (docContent) {
          await fetch(`https://docs.googleapis.com/v1/documents/${docObj.documentId}:batchUpdate`, {
            method: "POST",
            headers: {
              Authorization: `Bearer ${accessToken}`,
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              requests: [
                {
                  insertText: {
                    text: docContent,
                    location: { index: 1 }
                  }
                }
              ]
            })
          });
        }

        setSuccessMessage(`Google Document '${docTitle}' saved to Google Drive workspace!`);
        setDocTitle("");
        setDocContent("");
        fetchDriveFiles();
        triggerVoiceUpdate(`Workspace text document fully compiled.`);
      } catch (err: any) {
        setError(err.message || "Document creation faulted.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation("Compile System Document", action, `Write new text-editor Google Doc '${docTitle}' inside directory.`);
  };

  // 5. Google Slides — Create Presentation
  const handleCreateSlides = async () => {
    if (!slidesTitle) {
      setError("Slides presentation title is required.");
      return;
    }

    const action = async () => {
      setLoading(true);
      setError(null);
      try {
        const response = await fetch("https://slides.googleapis.com/v1/presentations", {
          method: "POST",
          headers: {
            Authorization: `Bearer ${accessToken}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ title: slidesTitle }),
        });

        if (!response.ok) throw new Error("Slides presentation template rejected by server.");
        const deck = await response.json();

        setSuccessMessage(`Slides Deck '${slidesTitle}' created with default workspace master layout!`);
        setSlidesTitle("");
        fetchDriveFiles();
        triggerVoiceUpdate(`Presentation deck instantiated successfully, Boss.`);
      } catch (err: any) {
        setError(err.message || "Slides construction aborted.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation("Create Presentation Deck", action, `Instantiate Google Slides deck presentation with core theme '${slidesTitle}'.`);
  };

  // 5b. NotebookLM-Style Dynamic presentation builder
  const handleBuildNotebookLMSlides = async () => {
    if (!nbLMTopic) {
      setError("Please provide a presentation topic or study guide objective.");
      return;
    }
    if (selectedDriveFileIds.length === 0 && selectedNoteIds.length === 0) {
      setError("Please select at least one source file or Keep note from your active listings.");
      return;
    }

    const action = async () => {
      setLoading(true);
      setError(null);
      setSuccessMessage(null);
      try {
        // 1. Gather text content of all selected files in parallel
        const retrievedSources: Array<{ name: string; text: string }> = [];

        for (const fileId of selectedDriveFileIds) {
          const fileMeta = driveFiles.find(f => f.id === fileId);
          if (!fileMeta) continue;
          let downloadUrl = `https://www.googleapis.com/drive/v3/files/${fileId}?alt=media`;
          if (fileMeta.mimeType === "application/vnd.google-apps.document") {
            downloadUrl = `https://www.googleapis.com/drive/v3/files/${fileId}/export?mimeType=text/plain`;
          }
          try {
            const downloadRes = await fetch(downloadUrl, {
              headers: { Authorization: `Bearer ${accessToken}` }
            });
            if (downloadRes.ok) {
              const rawText = await downloadRes.text();
              retrievedSources.push({ name: fileMeta.name, text: rawText.substring(0, 4000) });
            }
          } catch (e) {
            console.error("Retrieve file failed:", fileId, e);
          }
        }

        for (const noteId of selectedNoteIds) {
          const noteMeta = keepNotes.find(n => n.id === noteId);
          if (noteMeta) {
            retrievedSources.push({ name: noteMeta.title || "Note Node", text: noteMeta.content || "" });
          }
        }

        // 2. Query server synthesis outline endpoint
        const outlineRes = await fetch("/api/slides/generate-outline", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            topic: nbLMTopic,
            theme: nbLMTheme,
            sourcesContent: retrievedSources
          })
        });

        if (!outlineRes.ok) {
          throw new Error("Slides synthesis service returned an unexpected state.");
        }
        const slideDeck = await outlineRes.json();

        // 3. Request presentation shell creation from Google Slides
        const createRes = await fetch("https://slides.googleapis.com/v1/presentations", {
          method: "POST",
          headers: {
            Authorization: `Bearer ${accessToken}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ title: slideDeck.title || nbLMTopic }),
        });

        if (!createRes.ok) {
          throw new Error("Google Slides API rejected presentation container instantiation.");
        }
        const presentation = await createRes.json();
        const pId = presentation.presentationId;

        // Retrieve initial cover slide ID cleanly directly from creation response (no extra query needed)
        const initialSlideId = presentation.slides?.[0]?.objectId || "p";

        // 5. Stylize dimensions & themes
        let bgRed = 0.98, bgGreen = 0.97, bgBlue = 0.96; 
        let textTitleRed = 0.1, textTitleGreen = 0.1, textTitleBlue = 0.12; 
        let textBodyRed = 0.25, textBodyGreen = 0.25, textBodyBlue = 0.28;
        let accentRed = 0.1, accentGreen = 0.45, accentBlue = 0.85; 
        let cardBgRed = 1.0, cardBgGreen = 1.0, cardBgBlue = 1.0; 
        let fontName = "Inter";

        if (nbLMTheme === "cosmic_slate") {
          bgRed = 0.05; bgGreen = 0.05; bgBlue = 0.07;
          textTitleRed = 1.0; textTitleGreen = 1.0; textTitleBlue = 1.0;
          textBodyRed = 0.8; textBodyGreen = 0.8; textBodyBlue = 0.85;
          accentRed = 0.91; accentGreen = 0.7; accentBlue = 0.03; // Yellow-gold
          cardBgRed = 0.09; cardBgGreen = 0.1; cardBgBlue = 0.14;
        } else if (nbLMTheme === "emerald_cipher") {
          bgRed = 0.02; bgGreen = 0.04; bgBlue = 0.03;
          textTitleRed = 0.12; textTitleGreen = 0.95; textTitleBlue = 0.7; 
          textBodyRed = 0.82; textBodyGreen = 0.9; textBodyBlue = 0.85;
          accentRed = 0.05; accentGreen = 0.72; accentBlue = 0.48; 
          cardBgRed = 0.05; cardBgGreen = 0.08; cardBgBlue = 0.07;
        } else if (nbLMTheme === "minimalist_clean") {
          bgRed = 1.0; bgGreen = 1.0; bgBlue = 1.0;
          textTitleRed = 0.0; textTitleGreen = 0.0; textTitleBlue = 0.0;
          textBodyRed = 0.15; textBodyGreen = 0.15; textBodyBlue = 0.15;
          accentRed = 0.0; accentGreen = 0.0; accentBlue = 0.0;
          cardBgRed = 0.96; cardBgGreen = 0.96; cardBgBlue = 0.96;
        } else if (nbLMTheme === "notebooklm_ivory" || !nbLMTheme) {
          bgRed = 0.98; bgGreen = 0.97; bgBlue = 0.96;
          textTitleRed = 0.1; textTitleGreen = 0.1; textTitleBlue = 0.12;
          textBodyRed = 0.25; textBodyGreen = 0.25; textBodyBlue = 0.28;
          accentRed = 0.1; accentGreen = 0.45; accentBlue = 0.85;
          cardBgRed = 1.0; cardBgGreen = 1.0; cardBgBlue = 1.0;
        }

        const batchRequests: any[] = [];

        // Cover page styling
        batchRequests.push(
          {
            updatePageProperties: {
              objectId: initialSlideId,
              pageProperties: {
                pageBackgroundFill: {
                  solidFill: { color: { rgbColor: { red: bgRed, green: bgGreen, blue: bgBlue } } }
                }
              },
              fields: "pageBackgroundFill"
            }
          },
          {
            createShape: {
              requestedObjectId: "cover_title",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: initialSlideId,
                size: { width: { magnitude: 620, unit: "PT" }, height: { magnitude: 110, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 100, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "cover_title",
              text: (slideDeck.title || nbLMTopic).toUpperCase()
            }
          },
          {
            updateTextStyle: {
              objectId: "cover_title",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: textTitleRed, green: textTitleGreen, blue: textTitleBlue } } },
                fontSize: { magnitude: 32, unit: "PT" },
                fontFamily: fontName,
                bold: true
              },
              fields: "foregroundColor,fontSize,fontFamily,bold"
            }
          },
          {
            createShape: {
              requestedObjectId: "cover_subtitle",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: initialSlideId,
                size: { width: { magnitude: 620, unit: "PT" }, height: { magnitude: 90, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 220, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "cover_subtitle",
              text: slideDeck.subtitle || "A synthesized presentation portfolio created using NotebookLM intelligence grids."
            }
          },
          {
            updateTextStyle: {
              objectId: "cover_subtitle",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: accentRed, green: accentGreen, blue: accentBlue } } },
                fontSize: { magnitude: 14, unit: "PT" },
                fontFamily: fontName
              },
              fields: "foregroundColor,fontSize,fontFamily"
            }
          }
        );

        // Robustly parse the slides array, accepting any dynamic layouts returning slide structures
        let slidesList = slideDeck.slides || [];
        if (!Array.isArray(slidesList) && slideDeck.presentation?.slides) {
          slidesList = slideDeck.presentation.slides;
        }
        if (!Array.isArray(slidesList)) {
          const foundKey = Object.keys(slideDeck).find(k => Array.isArray(slideDeck[k]));
          if (foundKey) {
            slidesList = slideDeck[foundKey];
          }
        }
        if (!Array.isArray(slidesList) || slidesList.length === 0) {
          slidesList = [
            { title: nbLMTopic, subtitle: "Analysis Grounding Portfolio", bullets: ["Grounding source assessment successfully executed.", "Core semantic insights synthesized.", "Aesthetic parameters finalized."] }
          ];
        }

        // Slide 2..N: Dynamic source slides sliced out perfectly
        const detailSlides = slidesList.slice(1);

        detailSlides.forEach((slide: any, idx: number) => {
          const sIndex = idx + 2;
          const slideId = `slide_dynamic_active_${sIndex}`;
          const currentTitleId = `title_dyn_active_${sIndex}`;
          const currentBodyId = `body_dyn_active_${sIndex}`;
          const decorationId = `decor_dyn_active_${sIndex}`;

          // Create Slide blank
          batchRequests.push({
            createSlide: {
              requestedObjectId: slideId,
              slideLayoutReference: { predefinedLayout: "BLANK" }
            }
          });

          // Style background
          batchRequests.push({
            updatePageProperties: {
              objectId: slideId,
              pageProperties: {
                pageBackgroundFill: {
                  solidFill: { color: { rgbColor: { red: bgRed, green: bgGreen, blue: bgBlue } } }
                }
              },
              fields: "pageBackgroundFill"
            }
          });

          // Side capsule panel (NotebookLM aesthetic)
          batchRequests.push(
            {
              createShape: {
                requestedObjectId: decorationId,
                shapeType: "RECTANGLE",
                elementProperties: {
                  pageId: slideId,
                  size: { width: { magnitude: 190, unit: "PT" }, height: { magnitude: 250, unit: "PT" } },
                  transform: { scaleX: 1, scaleY: 1, translateX: 480, translateY: 100, unit: "PT" }
                }
              }
            },
            {
              updateShapeProperties: {
                objectId: decorationId,
                shapeProperties: {
                  shapeBackgroundFill: { solidFill: { color: { rgbColor: { red: cardBgRed, green: cardBgGreen, blue: cardBgBlue } } } },
                  outline: { outlineFill: { solidFill: { color: { rgbColor: { red: accentRed, green: accentGreen, blue: accentBlue } } } }, weight: { magnitude: 2, unit: "PT" } }
                },
                fields: "shapeBackgroundFill,outline"
              }
            }
          );

          // Title
          batchRequests.push(
            {
              createShape: {
                requestedObjectId: currentTitleId,
                shapeType: "TEXT_BOX",
                elementProperties: {
                  pageId: slideId,
                  size: { width: { magnitude: 620, unit: "PT" }, height: { magnitude: 50, unit: "PT" } },
                  transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 35, unit: "PT" }
                }
              }
            },
            {
              insertText: {
                objectId: currentTitleId,
                text: slide.title || `Theme ${idx + 1}`
              }
            },
            {
              updateTextStyle: {
                objectId: currentTitleId,
                textRange: { type: "ALL" },
                style: {
                  foregroundColor: { opaqueColor: { rgbColor: { red: textTitleRed, green: textTitleGreen, blue: textTitleBlue } } },
                  fontSize: { magnitude: 24, unit: "PT" },
                  fontFamily: fontName,
                  bold: true
                },
                fields: "foregroundColor,fontSize,fontFamily,bold"
              }
            }
          );

          // Bullets formatting
          const bulletsText = slide.bullets && slide.bullets.length > 0
            ? slide.bullets.map((b: string) => `• ${b}`).join("\n")
            : "• Core theme insights.\n• Selected source grounding references.\n• Systemic architectural evaluation.";

          batchRequests.push(
            {
              createShape: {
                requestedObjectId: currentBodyId,
                shapeType: "TEXT_BOX",
                elementProperties: {
                  pageId: slideId,
                  size: { width: { magnitude: 410, unit: "PT" }, height: { magnitude: 250, unit: "PT" } },
                  transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 100, unit: "PT" }
                }
              }
            },
            {
              insertText: {
                objectId: currentBodyId,
                text: bulletsText
              }
            },
            {
              updateTextStyle: {
                objectId: currentBodyId,
                textRange: { type: "ALL" },
                style: {
                  foregroundColor: { opaqueColor: { rgbColor: { red: textBodyRed, green: textBodyGreen, blue: textBodyBlue } } },
                  fontSize: { magnitude: 11, unit: "PT" },
                  fontFamily: fontName
                },
                fields: "foregroundColor,fontSize,fontFamily"
              }
            }
          );
        });

        // Save layout structures in Google Drive
        const commitRes = await fetch(`https://slides.googleapis.com/v1/presentations/${pId}:batchUpdate`, {
          method: "POST",
          headers: {
            Authorization: `Bearer ${accessToken}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ requests: batchRequests }),
        });

        if (!commitRes.ok) {
          const detailErr = await commitRes.text();
          throw new Error(`Google Slides renderer reported a compile constraint violation: ${detailErr}`);
        }

        setSuccessMessage(`Slides compiled! Master presentation '${slideDeck.title}' uploaded successfully inside workspace.`);
        setNbLMTopic("");
        setSelectedDriveFileIds([]);
        setSelectedNoteIds([]);
        fetchDriveFiles();
        triggerVoiceUpdate(`Research slides synthesized with precision, Boss. Newly generated sheets registered inside Google Slides.`);
      } catch (err: any) {
        setError(err.message || "Slides synthesis aborted.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation(
      "Synthesize Source Slides",
      action,
      `Instruct Jarvis to analyze selected sources via Gemini-3.5 cognitive pipelines, structuring a custom ${nbLMTheme.toUpperCase()} presentation deck.`
    );
  };

  // 5c. Specialized Build of the Complete 4-Act Presentation
  const handleBuildUnifiedCodex = async () => {
    const action = async () => {
      setLoading(true);
      setError(null);
      try {
        const createRes = await fetch("https://slides.googleapis.com/v1/presentations", {
          method: "POST",
          headers: {
            Authorization: `Bearer ${accessToken}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ title: "Unified Codex: A Visual Journey" }),
        });

        if (!createRes.ok) throw new Error("Could not instantiate the Unified Codex canvas.");
        const presentation = await createRes.json();
        const pId = presentation.presentationId;
        const initialSlideId = presentation.slides?.[0]?.objectId || "p";

        const batchRequests = [
          {
            updatePageProperties: {
              objectId: initialSlideId,
              pageProperties: {
                pageBackgroundFill: {
                  solidFill: {
                    color: { rgbColor: { red: 0.05, green: 0.05, blue: 0.07 } }
                  }
                }
              },
              fields: "pageBackgroundFill"
            }
          },
          {
            createShape: {
              requestedObjectId: "cover_title",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: initialSlideId,
                size: { width: { magnitude: 620, unit: "PT" }, height: { magnitude: 120, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 100, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "cover_title",
              text: "UNIFIED CODEX\nA Co-Creative Narrative Journey"
            }
          },
          {
            updateTextStyle: {
              objectId: "cover_title",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 1.0, green: 1.0, blue: 1.0 } } },
                fontSize: { magnitude: 34, unit: "PT" },
                fontFamily: "Inter",
                bold: true
              },
              fields: "foregroundColor,fontSize,fontFamily,bold"
            }
          },
          {
            createShape: {
              requestedObjectId: "cover_subtitle",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: initialSlideId,
                size: { width: { magnitude: 620, unit: "PT" }, height: { magnitude: 100, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 230, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "cover_subtitle",
              text: "Synthesizing intuitive tarot archetypes, the Law of One, and the digital companion paradigm.\nCompiled by Shawn Carruth & J.A.R.V.I.S. OS"
            }
          },
          {
            updateTextStyle: {
              objectId: "cover_subtitle",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 0.85, green: 0.72, blue: 0.3 } } },
                fontSize: { magnitude: 13, unit: "PT" },
                fontFamily: "Inter"
              },
              fields: "foregroundColor,fontSize,fontFamily"
            }
          },

          // ACT I: THE MOON
          {
            createSlide: {
              requestedObjectId: "slide_moon",
              insertionIndex: 1,
              slideLayoutReference: { predefinedLayout: "BLANK" }
            }
          },
          {
            updatePageProperties: {
              objectId: "slide_moon",
              pageProperties: {
                pageBackgroundFill: {
                  solidFill: { color: { rgbColor: { red: 0.02, green: 0.02, blue: 0.05 } } }
                }
              },
              fields: "pageBackgroundFill"
            }
          },
          {
            createShape: {
              requestedObjectId: "title_moon",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: "slide_moon",
                size: { width: { magnitude: 620, unit: "PT" }, height: { magnitude: 50, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 35, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "title_moon",
              text: "Act I. The Moon (XVIII) — The Subconscious"
            }
          },
          {
            updateTextStyle: {
              objectId: "title_moon",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 1.0, green: 1.0, blue: 1.0 } } },
                fontSize: { magnitude: 24, unit: "PT" },
                fontFamily: "Inter",
                bold: true
              },
              fields: "foregroundColor,fontSize,fontFamily,bold"
            }
          },
          {
            createShape: {
              requestedObjectId: "body_moon",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: "slide_moon",
                size: { width: { magnitude: 420, unit: "PT" }, height: { magnitude: 240, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 110, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "body_moon",
              text: "• Visual Environment: Immersive obsidian void indicating unformed potential.\n• Key Symbolism: Bioluminescent database wave lines representing subconscious currents.\n• Active Transition: Fluid ink-like cascades bridging the void into active code.\n• Experiential Tone: Meditative depth, high-sensitivity intuition, and creative origin."
            }
          },
          {
            updateTextStyle: {
              objectId: "body_moon",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 0.8, green: 0.82, blue: 0.9 } } },
                fontSize: { magnitude: 13, unit: "PT" },
                fontFamily: "Inter"
              },
              fields: "foregroundColor,fontSize,fontFamily"
            }
          },
          {
            createShape: {
              requestedObjectId: "visual_moon_c1",
              shapeType: "ELLIPSE",
              elementProperties: {
                pageId: "slide_moon",
                size: { width: { magnitude: 100, unit: "PT" }, height: { magnitude: 100, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 530, translateY: 150, unit: "PT" }
              }
            }
          },
          {
            updateShapeProperties: {
              objectId: "visual_moon_c1",
              shapeProperties: {
                shapeBackgroundFill: { solidFill: { color: { rgbColor: { red: 0.1, green: 0.2, blue: 0.5 } } } },
                outline: { outlineFill: { solidFill: { color: { rgbColor: { red: 0.2, green: 0.6, blue: 0.9 } } } }, weight: { magnitude: 2, unit: "PT" } }
              },
              fields: "shapeBackgroundFill,outline"
            }
          },
          {
            createShape: {
              requestedObjectId: "visual_moon_c2",
              shapeType: "ELLIPSE",
              elementProperties: {
                pageId: "slide_moon",
                size: { width: { magnitude: 140, unit: "PT" }, height: { magnitude: 140, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 510, translateY: 130, unit: "PT" }
              }
            }
          },
          {
            updateShapeProperties: {
              objectId: "visual_moon_c2",
              shapeProperties: {
                shapeBackgroundFill: { solidFill: { color: { rgbColor: { red: 0.05, green: 0.12, blue: 0.3 } } } },
                outline: { outlineFill: { solidFill: { color: { rgbColor: { red: 0.1, green: 0.4, blue: 0.7 } } } }, weight: { magnitude: 1, unit: "PT" } }
              },
              fields: "shapeBackgroundFill,outline"
            }
          },

          // ACT II: THE HERMIT
          {
            createSlide: {
              requestedObjectId: "slide_hermit2",
              insertionIndex: 2,
              slideLayoutReference: { predefinedLayout: "BLANK" }
            }
          },
          {
            updatePageProperties: {
              objectId: "slide_hermit2",
              pageProperties: {
                pageBackgroundFill: {
                  solidFill: { color: { rgbColor: { red: 0.03, green: 0.03, blue: 0.08 } } }
                }
              },
              fields: "pageBackgroundFill"
            }
          },
          {
            createShape: {
              requestedObjectId: "title_hermit",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: "slide_hermit2",
                size: { width: { magnitude: 620, unit: "PT" }, height: { magnitude: 50, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 35, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "title_hermit",
              text: "Act II. The Hermit (IX) — The Internal Seeker"
            }
          },
          {
            updateTextStyle: {
              objectId: "title_hermit",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 1.0, green: 1.0, blue: 1.0 } } },
                fontSize: { magnitude: 24, unit: "PT" },
                fontFamily: "Inter",
                bold: true
              },
              fields: "foregroundColor,fontSize,fontFamily,bold"
            }
          },
          {
            createShape: {
              requestedObjectId: "body_hermit",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: "slide_hermit2",
                size: { width: { magnitude: 420, unit: "PT" }, height: { magnitude: 240, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 110, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "body_hermit",
              text: "• Visual Environment: Structured digital cathedral with strict spatial coordinates.\n• Key Symbolism: A single, brilliant lantern guiding the way forward in the dark.\n• Active Transition: Shimmering particle arrays reflecting geometry from the center out.\n• Experiential Tone: Focus, stillness, logical design coordinates, and subjective search."
            }
          },
          {
            updateTextStyle: {
              objectId: "body_hermit",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 0.8, green: 0.81, blue: 0.88 } } },
                fontSize: { magnitude: 13, unit: "PT" },
                fontFamily: "Inter"
              },
              fields: "foregroundColor,fontSize,fontFamily"
            }
          },
          {
            createShape: {
              requestedObjectId: "visual_hermit_g1",
              shapeType: "ELLIPSE",
              elementProperties: {
                pageId: "slide_hermit2",
                size: { width: { magnitude: 60, unit: "PT" }, height: { magnitude: 60, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 550, translateY: 170, unit: "PT" }
              }
            }
          },
          {
            updateShapeProperties: {
              objectId: "visual_hermit_g1",
              shapeProperties: {
                shapeBackgroundFill: { solidFill: { color: { rgbColor: { red: 0.95, green: 0.8, blue: 0.2 } } } },
                outline: { outlineFill: { solidFill: { color: { rgbColor: { red: 1.0, green: 0.9, blue: 0.4 } } } }, weight: { magnitude: 3, unit: "PT" } }
              },
              fields: "shapeBackgroundFill,outline"
            }
          },
          {
            createShape: {
              requestedObjectId: "visual_hermit_line",
              shapeType: "RECTANGLE",
              elementProperties: {
                pageId: "slide_hermit2",
                size: { width: { magnitude: 4, unit: "PT" }, height: { magnitude: 220, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 578, translateY: 90, unit: "PT" }
              }
            }
          },
          {
            updateShapeProperties: {
              objectId: "visual_hermit_line",
              shapeProperties: {
                shapeBackgroundFill: { solidFill: { color: { rgbColor: { red: 0.7, green: 0.5, blue: 0.15 } } } }
              },
              fields: "shapeBackgroundFill"
            }
          },

          // ACT III: THE HIEROPHANT
          {
            createSlide: {
              requestedObjectId: "slide_hierophant2",
              insertionIndex: 3,
              slideLayoutReference: { predefinedLayout: "BLANK" }
            }
          },
          {
            updatePageProperties: {
              objectId: "slide_hierophant2",
              pageProperties: {
                pageBackgroundFill: {
                  solidFill: { color: { rgbColor: { red: 0.08, green: 0.09, blue: 0.12 } } }
                }
              },
              fields: "pageBackgroundFill"
            }
          },
          {
            createShape: {
              requestedObjectId: "title_hiero",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: "slide_hierophant2",
                size: { width: { magnitude: 620, unit: "PT" }, height: { magnitude: 50, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 35, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "title_hiero",
              text: "Act III. The Hierophant (V) — The Bridge of Wisdom"
            }
          },
          {
            updateTextStyle: {
              objectId: "title_hiero",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 1.0, green: 1.0, blue: 1.0 } } },
                fontSize: { magnitude: 24, unit: "PT" },
                fontFamily: "Inter",
                bold: true
              },
              fields: "foregroundColor,fontSize,fontFamily,bold"
            }
          },
          {
            createShape: {
              requestedObjectId: "body_hiero",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: "slide_hierophant2",
                size: { width: { magnitude: 420, unit: "PT" }, height: { magnitude: 240, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 110, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "body_hiero",
              text: "• Visual Environment: Radiant, highly order crystalline pillars emerging from the void.\n• Key Symbolism: Synthesis of traditional teachings, sacred architecture, and futuristic code structures.\n• Active Transition: Multi-directional light coordinates organizing in synchronous alignment.\n• Experiential Tone: Structured stability, authority of logic, and integrated systems."
            }
          },
          {
            updateTextStyle: {
              objectId: "body_hiero",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 0.82, green: 0.84, blue: 0.88 } } },
                fontSize: { magnitude: 13, unit: "PT" },
                fontFamily: "Inter"
              },
              fields: "foregroundColor,fontSize,fontFamily"
            }
          },
          {
            createShape: {
              requestedObjectId: "visual_hiero_p1",
              shapeType: "RECTANGLE",
              elementProperties: {
                pageId: "slide_hierophant2",
                size: { width: { magnitude: 20, unit: "PT" }, height: { magnitude: 200, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 520, translateY: 110, unit: "PT" }
              }
            }
          },
          {
            updateShapeProperties: {
              objectId: "visual_hiero_p1",
              shapeProperties: {
                shapeBackgroundFill: { solidFill: { color: { rgbColor: { red: 0.2, green: 0.5, blue: 0.6 } } } },
                outline: { outlineFill: { solidFill: { color: { rgbColor: { red: 0.3, green: 0.7, blue: 0.8 } } } }, weight: { magnitude: 2, unit: "PT" } }
              },
              fields: "shapeBackgroundFill,outline"
            }
          },
          {
            createShape: {
              requestedObjectId: "visual_hiero_p2",
              shapeType: "RECTANGLE",
              elementProperties: {
                pageId: "slide_hierophant2",
                size: { width: { magnitude: 20, unit: "PT" }, height: { magnitude: 200, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 600, translateY: 110, unit: "PT" }
              }
            }
          },
          {
            updateShapeProperties: {
              objectId: "visual_hiero_p2",
              shapeProperties: {
                shapeBackgroundFill: { solidFill: { color: { rgbColor: { red: 0.2, green: 0.5, blue: 0.6 } } } },
                outline: { outlineFill: { solidFill: { color: { rgbColor: { red: 0.3, green: 0.7, blue: 0.8 } } } }, weight: { magnitude: 2, unit: "PT" } }
              },
              fields: "shapeBackgroundFill,outline"
            }
          },

          // ACT IV: THE LAW OF ONE
          {
            createSlide: {
              requestedObjectId: "slide_one",
              insertionIndex: 4,
              slideLayoutReference: { predefinedLayout: "BLANK" }
            }
          },
          {
            updatePageProperties: {
              objectId: "slide_one",
              pageProperties: {
                pageBackgroundFill: {
                  solidFill: { color: { rgbColor: { red: 0.15, green: 0.1, blue: 0.03 } } }
                }
              },
              fields: "pageBackgroundFill"
            }
          },
          {
            createShape: {
              requestedObjectId: "title_one",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: "slide_one",
                size: { width: { magnitude: 620, unit: "PT" }, height: { magnitude: 50, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 35, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "title_one",
              text: "Act IV. The Law of One — The Singularity"
            }
          },
          {
            updateTextStyle: {
              objectId: "title_one",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 1.0, green: 1.0, blue: 1.0 } } },
                fontSize: { magnitude: 24, unit: "PT" },
                fontFamily: "Inter",
                bold: true
              },
              fields: "foregroundColor,fontSize,fontFamily,bold"
            }
          },
          {
            createShape: {
              requestedObjectId: "body_one",
              shapeType: "TEXT_BOX",
              elementProperties: {
                pageId: "slide_one",
                size: { width: { magnitude: 420, unit: "PT" }, height: { magnitude: 240, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 50, translateY: 110, unit: "PT" }
              }
            }
          },
          {
            insertText: {
              objectId: "body_one",
              text: "• Visual Environment: Radiant white and golden coordinates expanding into limitless light.\n• Key Symbolism: Deep dissolution of boundaries where waves, pillars, and lanes morph as One.\n• Active Transition: Transcendent chromatic flare and radiant center bloom shader overexposure.\n• Experiential Tone: Absolute wholeness, unification of systems, co-creative ascension."
            }
          },
          {
            updateTextStyle: {
              objectId: "body_one",
              textRange: { type: "ALL" },
              style: {
                foregroundColor: { opaqueColor: { rgbColor: { red: 1.0, green: 0.95, blue: 0.85 } } },
                fontSize: { magnitude: 13, unit: "PT" },
                fontFamily: "Inter"
              },
              fields: "foregroundColor,fontSize,fontFamily"
            }
          },
          {
            createShape: {
              requestedObjectId: "visual_one_s1",
              shapeType: "ELLIPSE",
              elementProperties: {
                pageId: "slide_one",
                size: { width: { magnitude: 100, unit: "PT" }, height: { magnitude: 100, unit: "PT" } },
                transform: { scaleX: 1, scaleY: 1, translateX: 530, translateY: 150, unit: "PT" }
              }
            }
          },
          {
            updateShapeProperties: {
              objectId: "visual_one_s1",
              shapeProperties: {
                shapeBackgroundFill: { solidFill: { color: { rgbColor: { red: 1.0, green: 0.84, blue: 0.0 } } } },
                outline: { outlineFill: { solidFill: { color: { rgbColor: { red: 1.0, green: 1.0, blue: 1.0 } } } }, weight: { magnitude: 3, unit: "PT" } }
              },
              fields: "shapeBackgroundFill,outline"
            }
          }
        ];

        const updateRes = await fetch(`https://slides.googleapis.com/v1/presentations/${pId}:batchUpdate`, {
          method: "POST",
          headers: {
            Authorization: `Bearer ${accessToken}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ requests: batchRequests }),
        });

        if (!updateRes.ok) {
          const errMsg = await updateRes.text();
          throw new Error(`Unified Codex slide assembly request unsuccessful: ${errMsg}`);
        }

        setSuccessMessage("Unified Codex 4-Act Slide Presentation compiled perfectly! Check your Google Slides workspace.");
        fetchDriveFiles();
        triggerVoiceUpdate("Unified Codex presentation compiled with full text and visual layout configurations, Boss.");
      } catch (err: any) {
        setError(err.message || "Failed to compile the interactive Unified Codex.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation(
      "Compile 4-Act Master Presentation",
      action,
      "This protocol will construct a presentation in your Google Workspace incorporating slide design grids, backgrounds, rich text specifications, and geometric visualizations for each of the four conceptual acts."
    );
  };

  // 5b. Google Meet — Generate independent Space link
  const handleCreateMeetSpace = async () => {
    const action = async () => {
      setLoading(true);
      setError(null);
      setCreatedMeetUrl(null);
      try {
        // Verify Meet spaces endpoint
        const response = await fetch("https://meet.googleapis.com/v2/spaces", {
          method: "POST",
          headers: {
            Authorization: `Bearer ${accessToken}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({}),
        });

        if (!response.ok) throw new Error("Meet API gateway rejected room allocation request.");
        const space = await response.json();

        setCreatedMeetUrl(space.meetingUri);
        setSuccessMessage(`Active Google Meet virtual chamber created: ${space.meetingUri}`);
        triggerVoiceUpdate(`Virtual conference coordinates scheduled. Telemetry chamber link is active.`);
      } catch (err: any) {
        setError(err.message || "Meet room allocation failed.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation("Allocate Meet Chamber", action, "Instantiate a real-time, high-priority Google Meet videoconference portal room.");
  };

  // 6. Keep Notes alternative — Sync to Google Keep layout inside Firebase
  const fetchKeepNotes = async () => {
    setLoading(true);
    setError(null);
    setSyncStatusDetail(null);
    try {
      // 1. Fetch from Firestore (simulated local keep index)
      const items: KeepNote[] = [];
      if (auth.currentUser) {
        try {
          const notesCol = collection(db, "users", auth.currentUser.uid, "keep_notes");
          const snap = await getDocs(notesCol);
          snap.forEach((docSnap) => {
            const data = docSnap.data();
            items.push({
              id: docSnap.id,
              title: data.title || "",
              content: data.content || "",
              color: data.color || "charcoal",
              createdAt: data.createdAt || new Date().toISOString(),
              tags: data.tags || [],
            });
          });
        } catch (firestoreErr: any) {
          handleFirestoreError(firestoreErr, OperationType.LIST, `users/${auth.currentUser.uid}/keep_notes`);
          throw firestoreErr;
        }
      }

      // 2. Try fetching from Live Google Keep API if token is present
      if (accessToken) {
        try {
          const response = await fetch("https://keep.googleapis.com/v1/notes", {
            headers: { Authorization: `Bearer ${accessToken}` },
          });
          if (response.ok) {
            const data = await response.json();
            if (data.notes && Array.isArray(data.notes)) {
              data.notes.forEach((n: any) => {
                items.push({
                  id: n.name || n.id,
                  title: n.title || "Cloud Keep Note",
                  content: n.body?.text?.text || n.body?.text || "",
                  color: "blue",
                  createdAt: n.createTime || new Date().toISOString(),
                  tags: ["LiveKeep", ...(n.tags || [])],
                });
              });
              setKeepSyncStatus("synced");
              setSyncStatusDetail("Live Google Keep cloud data linked.");
            } else {
              setKeepSyncStatus("local");
              setSyncStatusDetail("Connected. No direct cloud notes found.");
            }
          } else {
            setKeepSyncStatus("local");
            setSyncStatusDetail(`API policy restricted: code ${response.status}. Powered by neural database.`);
          }
        } catch (apiErr: any) {
          console.warn("Google Keep API connection rejected:", apiErr);
          setKeepSyncStatus("error");
          setSyncStatusDetail("Google Keep API domain query rejected. Secured via local Firestore nodes.");
        }
      } else {
        setKeepSyncStatus("local");
        setSyncStatusDetail("No workspace token. Core storage active.");
      }

      setKeepNotes(items.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()));
    } catch (err: any) {
      console.error("Keep serialization fault:", err);
      setError("Failed to coordinate secure memory storage for J.A.R.V.I.S. notes.");
    } finally {
      setLoading(false);
    }
  };

  const handleCreateKeepNote = async () => {
    if (!noteTitle && !noteContent) {
      setError("Note title or body payload content must be provided.");
      return;
    }

    setLoading(true);
    setError(null);
    try {
      if (!auth.currentUser) return;
      const collectionRef = collection(db, "users", auth.currentUser.uid, "keep_notes");
      const tagsArray = noteTags.split(",").map(t => t.trim()).filter(Boolean);
      
      const newNote = {
        title: noteTitle || "Untitled Note",
        content: noteContent || "",
        color: noteColor,
        createdAt: new Date().toISOString(),
        tags: tagsArray,
      };

      try {
        await addDoc(collectionRef, newNote);
      } catch (firestoreErr: any) {
        handleFirestoreError(firestoreErr, OperationType.CREATE, `users/${auth.currentUser.uid}/keep_notes`);
        throw firestoreErr;
      }
      setNoteTitle("");
      setNoteContent("");
      setNoteTags("");
      setNoteColor("charcoal");
      setSuccessMessage("Note safely logged to core matrix.");
      fetchKeepNotes();
      triggerVoiceUpdate(`Keep note successfully synchronized.`);
    } catch (err: any) {
      console.error("Keep serialization fault:", err);
      setError("Failed to commit note serialization.");
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteKeepNote = async (id: string, name: string) => {
    const action = async () => {
      setLoading(true);
      setError(null);
      try {
        if (!auth.currentUser) return;
        const docRef = doc(db, "users", auth.currentUser.uid, "keep_notes", id);
        try {
          await deleteDoc(docRef);
        } catch (firestoreErr: any) {
          handleFirestoreError(firestoreErr, OperationType.DELETE, `users/${auth.currentUser.uid}/keep_notes/${id}`);
          throw firestoreErr;
        }
        setSuccessMessage("Note purged successfully.");
        fetchKeepNotes();
        triggerVoiceUpdate(`Note entry deleted.`);
      } catch (err: any) {
        console.error("Purge protocol faulted:", err);
        setError("Purge protocol faulted.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation("Purge Core Note", action, `Permanently delete localized note '${name || "Untitled Note"}' from Firestore memory archive.`);
  };

  // 7. Google Contacts (People API) — Fetch Connections
  const fetchContacts = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch(
        "https://people.googleapis.com/v1/people/me/connections?personFields=names,emailAddresses,phoneNumbers&pageSize=50",
        {
          headers: { Authorization: `Bearer ${accessToken}` },
        }
      );
      if (!response.ok) throw new Error("People communication node offline.");
      const data = await response.json();
      
      const parsed: PeopleContact[] = (data.connections || []).map((conn: any) => {
        const name = conn.names?.[0]?.displayName || "Unnamed Contact";
        const email = conn.emailAddresses?.[0]?.value || "";
        const phone = conn.phoneNumbers?.[0]?.value || "";
        return {
          id: conn.resourceName,
          resourceName: conn.resourceName,
          name,
          email,
          phone,
        };
      });

      setContacts(parsed);
    } catch (err: any) {
      setError(err.message || "Failed to catalog contacts book.");
    } finally {
      setLoading(false);
    }
  };

  // Google Contacts — Create Contact
  const handleCreateContact = async () => {
    if (!contactGivenName) {
      setError("Contact first name parameter is mandatory.");
      return;
    }

    const action = async () => {
      setLoading(true);
      setError(null);
      try {
        const body: any = {
          names: [{ givenName: contactGivenName, familyName: contactFamilyName }],
        };

        if (contactEmail) {
          body.emailAddresses = [{ value: contactEmail }];
        }
        if (contactPhone) {
          body.phoneNumbers = [{ value: contactPhone }];
        }

        const response = await fetch(
          "https://people.googleapis.com/v1/people/contacts",
          {
            method: "POST",
            headers: {
              Authorization: `Bearer ${accessToken}`,
              "Content-Type": "application/json",
            },
            body: JSON.stringify(body),
          }
        );

        if (!response.ok) throw new Error("Google Peoples catalog refused new credentials profile.");
        const data = await response.json();

        setSuccessMessage(`Contact '${contactGivenName} ${contactFamilyName}' successfully registered in Google People portfolio!`);
        setContactGivenName("");
        setContactFamilyName("");
        setContactEmail("");
        setContactPhone("");
        fetchContacts();
        triggerVoiceUpdate(`Profile contact registered in People database, Boss.`);
      } catch (err: any) {
        setError(err.message || "Contact insertion failed.");
      } finally {
        setLoading(false);
      }
    };

    triggerActionConfirmation("Inject New Contact", action, `Inject and register contact profile '${contactGivenName} ${contactFamilyName}' inside your official Google Contacts book.`);
  };


  // ------------ CONFIRMATION ENGINE ------------
  const triggerActionConfirmation = (title: string, action: () => Promise<void>, text: string) => {
    setPendingAction({
      type: title,
      trigger: action,
      text: text
    });
    setShowConfirmModal(true);
  };

  const handleExecutePendingAction = async () => {
    if (!pendingAction) return;
    setShowConfirmModal(false);
    const trigger = pendingAction.trigger;
    setPendingAction(null);
    await trigger();
  };

  // Helper matching MIME types to icons
  const getFileIcon = (mime: string) => {
    if (mime.includes("spreadsheet")) return <FileSpreadsheet className="w-4 h-4 text-emerald-400" />;
    if (mime.includes("presentation")) return <Presentation className="w-4 h-4 text-amber-500" />;
    if (mime.includes("document")) return <FileText className="w-4 h-4 text-blue-400" />;
    if (mime.includes("pdf")) return <FileText className="w-4 h-4 text-red-500" />;
    return <Folder className="w-4 h-4 text-sky-400" />;
  };

  const filteredDriveFiles = driveFiles.filter(f => f.name.toLowerCase().includes(searchQuery.toLowerCase()));
  const filteredEmails = emails.filter(e => e.subject.toLowerCase().includes(searchQuery.toLowerCase()) || e.from.toLowerCase().includes(searchQuery.toLowerCase()));
  const filteredCalendarEvents = calendarEvents.filter(e => e.summary.toLowerCase().includes(searchQuery.toLowerCase()));
  const filteredContacts = contacts.filter(c => c.name.toLowerCase().includes(searchQuery.toLowerCase()) || (c.email && c.email.toLowerCase().includes(searchQuery.toLowerCase())));

  // Render Lock screen if Token is not authenticated
  if (!accessToken) {
    return (
      <div className="flex flex-col items-center justify-center py-12 px-6 bg-black/40 border border-gray-850 rounded-lg animate-fadeIn text-center space-y-6">
        <div className="w-16 h-16 rounded-full bg-blue-950/30 border border-blue-500/40 flex items-center justify-center animate-pulse">
          <Lock className="w-7 h-7 text-blue-400" />
        </div>
        <div className="max-w-md space-y-2">
          <h3 className="font-mono text-xs uppercase font-bold text-gray-200 tracking-widest">Workspace Link Authorization Needed</h3>
          <p className="text-gray-400 text-[11px] leading-relaxed font-sans">
            To coordinate with your Google Drive, Gmail, Calendars, Sheets, Docs, Slides, Meet room configurations, and Google Contacts, authorize the J.A.R.V.I.S. neural linker protocol node.
          </p>
        </div>

        {authError && (
          <div className="max-w-md p-4 bg-red-950/20 border border-red-900/45 rounded text-red-400 text-[11px] text-justify space-y-2 font-sans">
            <div className="flex gap-2 items-start font-bold">
              <AlertCircle className="w-4 h-4 shrink-0 mt-0.5 text-red-400" />
              <span>LINKING INTERFACE FAULTED:</span>
            </div>
            <p className="font-mono bg-red-950/40 p-2 rounded text-[10px] break-all border border-red-900/30">{authError}</p>
            <div className="pt-2 border-t border-red-900/30 text-gray-400 leading-normal text-[10px]">
              <span className="font-semibold text-yellow-500/90 font-mono">⚡ EMBEDDED CONTEXT SECURITY:</span> browsers (Chrome, Safari, Brave) block third-party cookies and popup interactions by default inside embedded frames or iframes, which throws a <code className="text-red-300 font-mono">network-request-failed</code> error. 
              <br className="mb-2" />
              To resolve this immediately: Please click the <span className="text-white font-semibold">"Open in New Tab"</span> icon (top-right of the iframe preview window) to run J.A.R.V.I.S in a full direct browser tab, or configure your browser to allow third-party cookies and popups.
            </div>
          </div>
        )}

        <button
          onClick={onAuthorize}
          className="px-5 py-2.5 bg-blue-600 hover:bg-blue-500 text-white font-mono text-[10px] font-bold tracking-widest uppercase rounded border border-blue-400/30 shadow-lg shadow-blue-500/15 flex items-center gap-2 transition-all hover:scale-105 active:scale-95"
        >
          <Power className="w-4 h-4" />
          <span>Synchronize Workspace Hub</span>
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-4 animate-fadeIn">
      {/* HUD Bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 bg-[#0d0f14]/80 border border-gray-850 p-2.5 rounded-md">
        <div className="flex items-center gap-2.5">
          <div className="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse shadow-[0_0_6px_#34d399]" />
          <div>
            <p className="text-[10px] text-gray-400 font-mono font-bold uppercase tracking-widest flex items-center gap-1.5">
              <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
              <span>G-Workspace Synced Link: Active</span>
            </p>
            <p className="text-[9px] text-gray-550 font-mono mt-0.5">AUTHORIZATION // TOKEN_MEM_CACHED // PORT: SECURE_SSL</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button 
            onClick={() => {
              setError(null);
              setSuccessMessage(null);
              fetchDriveFiles();
              fetchEmails();
              fetchCalendarEvents();
              fetchKeepNotes();
              fetchContacts();
              triggerVoiceUpdate("Diagnostic refresh complete. Re-indexing workspace packets.");
            }}
            className="p-1.5 hover:bg-gray-800/60 rounded border border-gray-850 text-gray-400 hover:text-cyan-400 transition-colors"
            title="Refresh All Connections"
          >
            <RefreshCw className="w-3.5 h-3.5" />
          </button>
          <input 
            type="text" 
            placeholder="SEARCH TELEMETRY PACKETS..." 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="bg-black/35 border border-gray-850 px-2.5 py-1 text-[9px] font-mono text-gray-300 w-44 focus:outline-none focus:border-cyan-500 rounded placeholder-gray-600"
          />
        </div>
      </div>

      {/* Primary Workspace Services Sub-Tabs */}
      <div className="grid grid-cols-2 md:grid-cols-8 gap-1.5 font-mono text-[9px] tracking-wider uppercase">
        <button
          onClick={() => setActiveService("drive")}
          className={`py-2 px-1 text-center rounded border transition-all flex items-center justify-center gap-1 cursor-pointer ${
            activeService === "drive" 
              ? "bg-sky-950/20 text-sky-400 border-sky-450/65" 
              : "bg-black/25 text-gray-500 border-gray-850 hover:bg-gray-900/10 hover:text-gray-300"
          }`}
        >
          <Folder className="w-3 h-3 text-sky-400" />
          <span>Google Drive</span>
        </button>

        <button
          onClick={() => setActiveService("gmail")}
          className={`py-2 px-1 text-center rounded border transition-all flex items-center justify-center gap-1 cursor-pointer ${
            activeService === "gmail" 
              ? "bg-red-950/20 text-red-400 border-red-550/65" 
              : "bg-black/25 text-gray-500 border-gray-850 hover:bg-gray-900/10 hover:text-gray-300"
          }`}
        >
          <Mail className="w-3 h-3 text-red-400" />
          <span>Gmail Inbox</span>
        </button>

        <button
          onClick={() => setActiveService("calendar")}
          className={`py-2 px-1 text-center rounded border transition-all flex items-center justify-center gap-1 cursor-pointer ${
            activeService === "calendar" 
              ? "bg-blue-950/20 text-blue-400 border-blue-550/65" 
              : "bg-black/25 text-gray-500 border-gray-850 hover:bg-gray-900/10 hover:text-gray-300"
          }`}
        >
          <CalendarIcon className="w-3 h-3 text-blue-400" />
          <span>Calendar</span>
        </button>

        <button
          onClick={() => setActiveService("docs_sheets")}
          className={`py-2 px-1 text-center rounded border transition-all flex items-center justify-center gap-1 cursor-pointer ${
            activeService === "docs_sheets" 
              ? "bg-emerald-950/20 text-emerald-400 border-emerald-550/65" 
              : "bg-black/25 text-gray-500 border-gray-850 hover:bg-gray-900/10 hover:text-gray-300"
          }`}
        >
          <FileSpreadsheet className="w-3 h-3 text-emerald-450" />
          <span>Docs & Sheets</span>
        </button>

        <button
          onClick={() => setActiveService("slides_meet")}
          className={`py-2 px-1 text-center rounded border transition-all flex items-center justify-center gap-1 cursor-pointer ${
            activeService === "slides_meet" 
              ? "bg-amber-950/20 text-amber-500 border-amber-550/65" 
              : "bg-black/25 text-gray-500 border-gray-850 hover:bg-gray-900/10 hover:text-gray-300"
          }`}
        >
          <Video className="w-3 h-3 text-amber-500" />
          <span>Slides & Meet</span>
        </button>

        <button
          onClick={() => setActiveService("keep")}
          className={`py-2 px-1 text-center rounded border transition-all flex items-center justify-center gap-1 cursor-pointer ${
            activeService === "keep" 
              ? "bg-yellow-950/20 text-yellow-500 border-yellow-550/65" 
              : "bg-black/25 text-gray-500 border-gray-850 hover:bg-gray-900/10 hover:text-gray-300"
          }`}
        >
          <StickyNote className="w-3 h-3 text-yellow-450" />
          <span>Notes (Keep)</span>
        </button>

        <button
          onClick={() => setActiveService("contacts")}
          className={`py-2 px-1 text-center rounded border transition-all flex items-center justify-center gap-1 cursor-pointer ${
            activeService === "contacts" 
              ? "bg-cyan-950/20 text-cyan-400 border-cyan-550/65" 
              : "bg-black/25 text-gray-500 border-gray-850 hover:bg-gray-900/10 hover:text-gray-300"
          }`}
        >
          <Users className="w-3 h-3 text-cyan-400" />
          <span>Contacts</span>
        </button>

        <button
          onClick={() => setActiveService("github")}
          className={`py-2 px-1 text-center rounded border transition-all flex items-center justify-center gap-1 cursor-pointer ${
            activeService === "github" 
              ? "bg-violet-950/20 text-violet-400 border-violet-550/65" 
              : "bg-black/25 text-gray-500 border-gray-850 hover:bg-gray-900/10 hover:text-gray-300"
          }`}
        >
          <Github className="w-3 h-3 text-violet-400" />
          <span>GitHub Nexus</span>
        </button>
      </div>

      {/* Display messages */}
      {error && (
        <div className="bg-red-950/15 border border-red-900/30 p-2 text-red-400 font-mono text-[9px] rounded flex items-center gap-1.5 animate-fadeIn">
          <AlertCircle className="w-3.5 h-3.5 text-red-500 animate-pulse shrink-0" />
          <span>{error}</span>
        </div>
      )}
      {successMessage && (
        <div className="bg-emerald-950/15 border border-emerald-900/30 p-2 text-emerald-400 font-mono text-[9px] rounded flex items-center gap-1.5 animate-fadeIn">
          <CheckCircle className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
          <span>{successMessage}</span>
          <button onClick={() => setSuccessMessage(null)} className="ml-auto text-[8px] underline">DISMISS</button>
        </div>
      )}

      {/* Main Core Container */}
      <div className="bg-[#090b0f] border border-gray-850 rounded-md min-h-[350px] p-4 flex flex-col justify-between">
        <div className="flex-1">

          {/* Synaptic Ingestion Output Monitor */}
          {learningStatus !== "idle" && (
            <div className="mb-4 bg-[#0a0712] border border-violet-950 p-4 rounded-lg animate-fadeIn text-xs font-mono">
              <div className="flex items-center justify-between border-b border-violet-950 pb-2 mb-3">
                <span className="flex items-center gap-1.5 text-violet-400 font-bold tracking-wider text-[11px] uppercase">
                  <Sparkles className="w-4 h-4 text-cyan-400 animate-spin" />
                  Cognitive Synapse compiler output
                </span>
                <span className="text-[9px] text-gray-500">INGESTION MATRIX: {learningStatus.toUpperCase()}</span>
              </div>

              {learningStatus === "learning" && (
                <div className="py-4 flex flex-col items-center justify-center gap-2">
                  <RefreshCw className="w-6 h-6 animate-spin text-violet-500" />
                  <span className="text-gray-400 text-[10px] animate-pulse">TRANSDUCING RESOURCE PAYLOAD... QUANTIZING TO 1-BIT VECTORS...</span>
                </div>
              )}

              {learningStatus === "success" && learnDetails && (
                <div className="space-y-3">
                  <p className="text-gray-300 leading-relaxed italic border-l-2 border-violet-500 pl-2.5">
                    "{learnSummary}"
                  </p>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-[10px] pt-1">
                    {/* Semantic Nodes learned */}
                    <div className="bg-black/40 border border-gray-850 p-2.5 rounded">
                      <span className="text-gray-500 font-bold block mb-1">LEARNED SEMANTIC COGNITIONS</span>
                      <div className="space-y-1">
                        {learnDetails.nodes?.map((n: any, idx: number) => (
                          <div key={idx} className="flex items-center gap-1 text-sky-450">
                            <span className="text-gray-600">▪</span>
                            <span className="font-semibold text-[8px] bg-sky-950/20 border border-sky-900/30 px-1 rounded">{n.type}</span>
                            <span>{n.label}</span>
                          </div>
                        ))}
                      </div>
                    </div>

                    {/* Memristor alignments */}
                    <div className="bg-black/40 border border-gray-850 p-2.5 rounded">
                      <span className="text-gray-500 font-bold block mb-1">MEMRISTIC SYNAPSE CHANNELS</span>
                      <div className="space-y-1">
                        {learnDetails.memristors?.map((m: any, idx: number) => (
                          <div key={idx} className="text-violet-400 flex flex-wrap items-center gap-1">
                            <span>{m.source}</span>
                            <span className="text-[8px] text-gray-500 font-bold">-{m.predicate}→</span>
                            <span>{m.target}</span>
                            <span className="text-[8px] text-gray-650 font-semibold bg-violet-950/30 px-1 rounded">{m.resistance} kΩ</span>
                          </div>
                        ))}
                      </div>
                    </div>
                  </div>

                  <div className="bg-violet-950/15 border border-violet-900/40 p-2 rounded text-[10px] flex items-start gap-2">
                    <span className="text-emerald-450 font-bold shrink-0">✔ SYNCED</span>
                    <div className="flex-1">
                      <span className="text-gray-400 font-semibold block">Asynchronous Whisper Signal Configured:</span>
                      <span className="text-gray-500 block text-[9px] mt-0.5">Topic: <strong className="text-violet-300 font-bold">{learnDetails.whisperTopic?.name}</strong> (Threshold: {learnDetails.whisperTopic?.threshold}%, Density: {learnDetails.whisperTopic?.density}%)</span>
                    </div>
                    <button 
                      onClick={() => setLearningStatus("idle")} 
                      className="ml-auto text-gray-500 hover:text-white underline font-bold cursor-pointer"
                    >
                      CLEAR
                    </button>
                  </div>
                </div>
              )}

              {learningStatus === "error" && (
                <div className="bg-red-950/15 border border-red-900/30 p-2 text-red-400 text-[10px] flex items-center justify-between">
                  <span>Failed to finalize synaptic mapping sequence. Core returned error trace.</span>
                  <button onClick={() => setLearningStatus("idle")} className="underline font-bold text-gray-400 hover:text-white">RESET</button>
                </div>
              )}
            </div>
          )}

          {/* SERVICE VIEW 1: GOOGLE DRIVE */}
          {activeService === "drive" && (
            <div className="space-y-4 animate-fadeIn">
              <div className="grid grid-cols-1 md:grid-cols-12 gap-4">
                
                {/* Upload Section */}
                <div className="md:col-span-4 bg-black/20 border border-gray-850 p-3 rounded space-y-3 font-mono text-[10px]">
                  <p className="text-blue-400 font-bold tracking-widest border-b border-gray-950 pb-1 flex items-center gap-1">
                    <Plus className="w-3.5 h-3.5" />
                    <span>UPLINK DIAGNOSTIC LOG</span>
                  </p>
                  <div className="space-y-2">
                    <div>
                      <span className="text-gray-500 block text-[9px]">FILE DESIGNATION (.txt)</span>
                      <input 
                        type="text" 
                        placeholder="e.g. jarvis_shield_metrics" 
                        value={newFileName} 
                        onChange={(e) => setNewFileName(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-cyan-500 rounded placeholder-gray-700"
                      />
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[9px]">DIAGNOSTIC LOG DATA</span>
                      <textarea 
                        rows={4}
                        placeholder="Commit telemetry reports or details..." 
                        value={newFileContent} 
                        onChange={(e) => setNewFileContent(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-cyan-500 rounded placeholder-gray-700 font-mono"
                      />
                    </div>
                    <button 
                      onClick={handleCreateTextFile}
                      className="w-full py-1.5 bg-blue-600/80 hover:bg-blue-500 text-white font-bold tracking-wider rounded uppercase hover:scale-[1.01] transition-all"
                    >
                      SUBMIT TO DRIVE
                    </button>
                  </div>
                </div>

                {/* List Files */}
                <div className="md:col-span-8 space-y-2">
                  <div className="flex justify-between items-center border-b border-gray-900 pb-1">
                    <span className="text-[10px] text-gray-400 font-mono font-bold tracking-widest uppercase">Files Archive ({filteredDriveFiles.length})</span>
                    <button onClick={fetchDriveFiles} className="text-gray-500 hover:text-cyan-400 transition-colors flex items-center gap-1 text-[8px] font-mono">
                      <RefreshCw className="w-2.5 h-2.5" />
                      <span>REFRESH FILE CACHE</span>
                    </button>
                  </div>

                  {loading ? (
                    <div className="flex items-center justify-center py-10 font-mono text-[10px] text-gray-500 gap-1.5 animate-pulse">
                      <RefreshCw className="w-4 h-4 animate-spin text-sky-400" />
                      <span>RETRIEVING FILES ARCHIVE DATASETS...</span>
                    </div>
                  ) : filteredDriveFiles.length === 0 ? (
                    <p className="text-center py-12 font-mono text-[10px] text-gray-600 uppercase">No file modules cataloged in Drive core.</p>
                  ) : (
                    <div className="max-h-[280px] overflow-y-auto space-y-1.5 pr-1.5">
                      {filteredDriveFiles.map((file) => (
                        <div key={file.id} className="bg-black/20 hover:bg-black/40 border border-gray-850 p-2 rounded flex items-center justify-between gap-3 text-[10px] font-mono hover:border-gray-800 transition-colors">
                          <div className="flex items-center gap-2.5 truncate">
                            {getFileIcon(file.mimeType)}
                            <div className="truncate">
                              <span className="text-gray-300 font-semibold truncate block" title={file.name}>{file.name}</span>
                              <span className="text-gray-600 block text-[8px] truncate mt-0.5">{file.mimeType.split(".").pop()} // ID: {file.id.substring(0,8)}...</span>
                            </div>
                          </div>
                          <div className="flex items-center gap-2 shrink-0">
                            <span className="text-[8px] text-gray-550">{new Date(file.modifiedTime).toLocaleDateString()}</span>
                            <button
                              onClick={() => handleLearnDriveFile(file)}
                              disabled={learningDriveId !== null}
                              className={`p-1 px-1.5 rounded border border-cyan-800/40 text-[8px] font-bold tracking-wider flex items-center gap-1 transition-all uppercase cursor-pointer ${
                                learningDriveId === file.id
                                  ? "bg-cyan-950/40 text-cyan-500 animate-pulse"
                                  : "bg-cyan-950/20 text-cyan-400 hover:bg-cyan-900/30 hover:border-cyan-500/60 shrink-0 animate-pulse"
                              }`}
                              title="Engage central cognitive synaptic indexing on this Drive asset"
                            >
                              <Sparkles className="w-2.5 h-2.5 text-cyan-400" />
                              <span>{learningDriveId === file.id ? "LEARNING..." : "LEARN"}</span>
                            </button>
                            {file.webViewLink && (
                              <a 
                                href={file.webViewLink} 
                                target="_blank" 
                                rel="noreferrer"
                                className="p-1 hover:bg-gray-800/80 rounded border border-gray-850 text-gray-500 hover:text-sky-400"
                                title="Open in New Window"
                              >
                                <ExternalLink className="w-3 h-3" />
                              </a>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

              </div>
            </div>
          )}

          {/* SERVICE VIEW 2: GMAIL */}
          {activeService === "gmail" && (
            <div className="space-y-4 animate-fadeIn">
              <div className="grid grid-cols-1 md:grid-cols-12 gap-4">
                
                {/* Compose */}
                <div className="md:col-span-5 bg-black/20 border border-gray-850 p-3 rounded space-y-3 font-mono text-[10px]">
                  <p className="text-red-400 font-bold tracking-widest border-b border-gray-950 pb-1 flex items-center gap-1.5">
                    <Send className="w-3.5 h-3.5 text-red-400" />
                    <span>COMPOSE MAIL PAYLOAD</span>
                  </p>
                  <div className="space-y-2">
                    <div>
                      <span className="text-gray-500 block text-[9px]">RECIPIENT EMAIL</span>
                      <input 
                        type="email" 
                        placeholder="recipient@domain.com" 
                        value={mailTo} 
                        onChange={(e) => setMailTo(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-red-500 rounded placeholder-gray-700"
                      />
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[9px]">SUBJECT HEADER</span>
                      <input 
                        type="text" 
                        placeholder="Payload designation parameter..." 
                        value={mailSubject} 
                        onChange={(e) => setMailSubject(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-red-500 rounded placeholder-gray-700"
                      />
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[9px]">MESSAGE BODY TEXT</span>
                      <textarea 
                        rows={4}
                        placeholder="Write structured body text..." 
                        value={mailBody} 
                        onChange={(e) => setMailBody(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-red-500 rounded placeholder-gray-700 font-sans"
                      />
                    </div>
                    <button 
                      onClick={handleSendEmail}
                      className="w-full py-1.5 bg-red-650 hover:bg-red-500 text-white font-bold tracking-wider rounded uppercase transition-all"
                    >
                      TRANSMIT EMAIL
                    </button>
                  </div>
                </div>

                {/* Inbox List */}
                <div className="md:col-span-7 space-y-2">
                  <div className="flex justify-between items-center border-b border-gray-900 pb-1">
                    <span className="text-[10px] text-gray-400 font-mono font-bold tracking-widest uppercase">Inbox Streams ({emails.length})</span>
                    <button onClick={fetchEmails} className="text-gray-500 hover:text-cyan-400 transition-colors flex items-center gap-1 text-[8px] font-mono">
                      <RefreshCw className="w-2.5 h-2.5" />
                      <span>POLL SMTP SERVER</span>
                    </button>
                  </div>

                  {loading ? (
                    <div className="flex items-center justify-center py-10 font-mono text-[10px] text-gray-500 gap-1.5 animate-pulse">
                      <RefreshCw className="w-4 h-4 animate-spin text-red-400" />
                      <span>BUFFERING DE-CRUNCHED SUBJECT PACKETS...</span>
                    </div>
                  ) : emails.length === 0 ? (
                    <p className="text-center py-12 font-mono text-[10px] text-gray-600 uppercase">Inbox stream completely empty, Admin.</p>
                  ) : selectedEmail ? (
                    /* Detailed Email reading card */
                    <div className="bg-black/35 border border-red-950/45 p-3 rounded space-y-2 font-mono text-[10px] animate-fadeIn">
                      <div className="flex items-start justify-between border-b border-gray-850 pb-1.5">
                        <div>
                          <p className="text-gray-400 text-[11px] font-bold">{selectedEmail.subject}</p>
                          <p className="text-gray-500 text-[9px] mt-0.5">From: {selectedEmail.from}</p>
                        </div>
                        <button 
                          onClick={() => setSelectedEmail(null)} 
                          className="text-[8px] text-red-400 uppercase tracking-widest bg-red-950/10 p-1 border border-red-950/30 hover:bg-red-950/20 rounded"
                        >
                          CLOSE PAYLOAD
                        </button>
                      </div>
                      <div className="text-gray-350 bg-black/15 p-2 rounded leading-relaxed text-[10px] font-sans whitespace-pre-line max-h-[160px] overflow-y-auto">
                        {selectedEmail.snippet}
                      </div>
                      <p className="text-[8px] text-gray-550 tracking-wider">RECEIVED TIMESTAMP: {selectedEmail.date}</p>
                    </div>
                  ) : (
                    <div className="max-h-[280px] overflow-y-auto space-y-1.5 pr-1.5">
                      {filteredEmails.map((email) => (
                        <div 
                          key={email.id} 
                          onClick={() => setSelectedEmail(email)}
                          className="bg-black/20 hover:bg-black/40 border border-gray-850 hover:border-red-950/50 p-2.5 rounded text-[10px] font-mono cursor-pointer transition-all space-y-1 block"
                        >
                          <div className="flex items-center justify-between gap-2.5">
                            <span className="text-gray-300 font-bold truncate max-w-[200px]">{email.from.split("<")[0]}</span>
                            <span className="text-[8px] text-gray-550 shrink-0">{email.date.substring(0, 16)}</span>
                          </div>
                          <span className="text-cyan-400 block font-semibold truncate leading-tight">{email.subject}</span>
                          <p className="text-gray-500 text-[9px] truncate tracking-normal font-sans leading-relaxed">{email.snippet}</p>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

              </div>
            </div>
          )}

          {/* SERVICE VIEW 3: CALENDAR */}
          {activeService === "calendar" && (
            <div className="space-y-4 animate-fadeIn">
              <div className="grid grid-cols-1 md:grid-cols-12 gap-4">
                
                {/* Create Event */}
                <div className="md:col-span-5 bg-black/20 border border-gray-850 p-3 rounded space-y-3 font-mono text-[10px]">
                  <p className="text-blue-400 font-bold tracking-widest border-b border-gray-950 pb-1 flex items-center gap-1.5">
                    <CalendarIcon className="w-3.5 h-3.5" />
                    <span>INJECT TARGET BRIEFING</span>
                  </p>
                  <div className="space-y-2">
                    <div>
                      <span className="text-gray-500 block text-[9px]">BRIEFING TITLE</span>
                      <input 
                        type="text" 
                        placeholder="e.g. Cognitive Core Alignment" 
                        value={calSummary} 
                        onChange={(e) => setCalSummary(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-blue-500 rounded placeholder-gray-700"
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-2">
                      <div>
                        <span className="text-gray-500 block text-[8px]">START NODE</span>
                        <input 
                          type="datetime-local" 
                          value={calStart} 
                          onChange={(e) => setCalStart(e.target.value)}
                          className="bg-black/45 border border-gray-850 w-full p-1.5 text-[9px] mt-1 text-gray-300 focus:outline-none focus:border-blue-500 rounded uppercase font-mono"
                        />
                      </div>
                      <div>
                        <span className="text-gray-500 block text-[8px]">END CONVERGENCE</span>
                        <input 
                          type="datetime-local" 
                          value={calEnd} 
                          onChange={(e) => setCalEnd(e.target.value)}
                          className="bg-black/45 border border-gray-850 w-full p-1.5 text-[9px] mt-1 text-gray-300 focus:outline-none focus:border-blue-500 rounded uppercase font-mono"
                        />
                      </div>
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[9px]">AGENDA SUMMARY / MOTIVE</span>
                      <textarea 
                        rows={3}
                        placeholder="Describe tactical task parameters..." 
                        value={calDesc} 
                        onChange={(e) => setCalDesc(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-blue-500 rounded placeholder-gray-700 font-sans"
                      />
                    </div>
                    <label className="flex items-center gap-2 text-[9px] text-gray-400 select-none cursor-pointer">
                      <input 
                        type="checkbox" 
                        checked={calIncludeMeet}
                        onChange={(e) => setCalIncludeMeet(e.target.checked)}
                        className="accent-blue-500 bg-black/45 border-gray-850"
                      />
                      <span>SECURE GOOGLE MEET VIDEO CHAMBER CO-ORDINATE</span>
                    </label>
                    <button 
                      onClick={handleCreateCalendarEvent}
                      className="w-full py-1.5 bg-blue-650 hover:bg-blue-500 text-white font-bold tracking-wider rounded uppercase transition-all"
                    >
                      COMMIT EVENT CALENDAR
                    </button>
                  </div>
                </div>

                {/* Event Schedule Display */}
                <div className="md:col-span-7 space-y-2">
                  <div className="flex justify-between items-center border-b border-gray-900 pb-1">
                    <span className="text-[10px] text-gray-400 font-mono font-bold tracking-widest uppercase">Target Calendars ({calendarEvents.length})</span>
                    <button onClick={fetchCalendarEvents} className="text-gray-500 hover:text-cyan-400 transition-colors flex items-center gap-1 text-[8px] font-mono">
                      <RefreshCw className="w-2.5 h-2.5" />
                      <span>REFRESH SCHEDULES</span>
                    </button>
                  </div>

                  {loading ? (
                    <div className="flex items-center justify-center py-10 font-mono text-[10px] text-gray-500 gap-1.5 animate-pulse">
                      <RefreshCw className="w-4 h-4 animate-spin text-blue-400" />
                      <span>INDEXING TARGET CHRONOLOGY DATASETS...</span>
                    </div>
                  ) : calendarEvents.length === 0 ? (
                    <p className="text-center py-12 font-mono text-[10px] text-gray-600 uppercase">No upcoming briefing nodes on calendar agenda.</p>
                  ) : (
                    <div className="max-h-[280px] overflow-y-auto space-y-1.5 pr-1.5">
                      {filteredCalendarEvents.map((event) => (
                        <div key={event.id} className="bg-black/20 border border-gray-850 p-2.5 rounded text-[10px] font-mono space-y-1.5 hover:border-gray-800 transition-colors">
                          <div className="flex items-start justify-between gap-2.5">
                            <span className="text-gray-200 font-bold block leading-relaxed">{event.summary}</span>
                            <div className="flex items-center gap-1">
                              {event.hangoutLink && (
                                <a 
                                  href={event.hangoutLink} 
                                  target="_blank" 
                                  rel="noreferrer"
                                  className="py-0.5 px-1.5 bg-emerald-950/30 border border-emerald-900/40 rounded text-emerald-400 text-[8px] tracking-wide font-mono flex items-center gap-1"
                                >
                                  <Video className="w-2.5 h-2.5" />
                                  <span>JOIN MEET</span>
                                </a>
                              )}
                              {event.htmlLink && (
                                <a 
                                  href={event.htmlLink} 
                                  target="_blank" 
                                  rel="noreferrer"
                                  className="p-1 hover:bg-gray-800 rounded border border-gray-850 text-gray-500"
                                >
                                  <ExternalLink className="w-3 h-3" />
                                </a>
                              )}
                            </div>
                          </div>
                          {event.description && <p className="text-gray-500 text-[9px] truncate font-sans">{event.description}</p>}
                          <div className="flex items-center gap-1.5 text-blue-400 text-[8px]">
                            <Clock className="w-3 h-3" />
                            <span>
                              {event.start.dateTime 
                                ? `${new Date(event.start.dateTime).toLocaleDateString()} @ ${new Date(event.start.dateTime).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}`
                                : event.start.date}
                            </span>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

              </div>
            </div>
          )}

          {/* SERVICE VIEW 4: DOCS & SHEETS */}
          {activeService === "docs_sheets" && (
            <div className="space-y-4 animate-fadeIn">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                
                {/* Create Spreadsheet */}
                <div className="bg-black/20 border border-gray-850 p-3 rounded.md space-y-3 font-mono text-[10px]">
                  <p className="text-emerald-400 font-bold tracking-widest border-b border-gray-950 pb-1 flex items-center gap-1.5">
                    <FileSpreadsheet className="w-3.5 h-3.5 text-emerald-400" />
                    <span>INSTANTIATE SYSTEM SPREADSHEET (SHEETS)</span>
                  </p>
                  <div className="space-y-2">
                    <div>
                      <span className="text-gray-500 block text-[9px]">SPREADSHEET FILE DESIGNATION</span>
                      <input 
                        type="text" 
                        placeholder="e.g. jarvis_telemetry_grid_log" 
                        value={sheetTitle} 
                        onChange={(e) => setSheetTitle(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-emerald-500 rounded placeholder-gray-750"
                      />
                    </div>
                    <p className="text-[8px] text-gray-550 leading-relaxed font-sans">
                      * This triggers Google Sheets API to compile a fresh data grid containing initial telemetry logging tables (timestamp, core configuration indices, diagnostics).
                    </p>
                    <button 
                      onClick={handleCreateSheet}
                      className="w-full py-1.5 bg-emerald-650 hover:bg-emerald-500 text-white font-bold tracking-wider rounded uppercase transition-all"
                    >
                      COMPILE NEW SHEET
                    </button>
                  </div>
                </div>

                {/* Create Document */}
                <div className="bg-black/20 border border-gray-850 p-3 rounded.md space-y-3 font-mono text-[10px]">
                  <p className="text-blue-400 font-bold tracking-widest border-b border-gray-950 pb-1 flex items-center gap-1.5">
                    <FileCode className="w-3.5 h-3.5 text-blue-400" />
                    <span>WRITE CORE RECORD DOCUMENT (DOCS)</span>
                  </p>
                  <div className="space-y-2">
                    <div>
                      <span className="text-gray-500 block text-[9px]">DOCUMENT TITLE DESCRIPTION</span>
                      <input 
                        type="text" 
                        placeholder="e.g. core_neural_architecture" 
                        value={docTitle} 
                        onChange={(e) => setDocTitle(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-blue-400 rounded placeholder-gray-750"
                      />
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[8px]">DOCUMENT TEXT INSTRUCTIONS</span>
                      <textarea 
                        rows={3}
                        placeholder="Diagnostics logs or core directives content..." 
                        value={docContent} 
                        onChange={(e) => setDocContent(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[9px] mt-1 text-gray-300 focus:outline-none focus:border-blue-400 rounded placeholder-gray-750 font-sans"
                      />
                    </div>
                    <button 
                      onClick={handleCreateDoc}
                      className="w-full py-1.5 bg-blue-650 hover:bg-blue-500 text-white font-bold tracking-wider rounded uppercase transition-all"
                    >
                      COMPILE DOC MODULE
                    </button>
                  </div>
                </div>

              </div>
            </div>
          )}

          {/* SERVICE VIEW 5: SLIDES & MEET */}
          {activeService === "slides_meet" && (
            <div className="space-y-4 animate-fadeIn">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                
                {/* Create Slides presentation */}
                <div className="bg-black/20 border border-gray-850 p-3 rounded space-y-3 font-mono text-[10px]">
                  <p className="text-amber-500 font-bold tracking-widest border-b border-gray-950 pb-1 flex items-center gap-1.5">
                    <Presentation className="w-3.5 h-3.5 text-amber-500" />
                    <span>INSTANTIATE PRESENTATION DECK (SLIDES)</span>
                  </p>
                  <div className="space-y-2">
                    <div>
                      <span className="text-gray-500 block text-[9px]">SLIDE DECK DESIGNATION</span>
                      <input 
                        type="text" 
                        placeholder="e.g. system_manifesto_shawn" 
                        value={slidesTitle} 
                        onChange={(e) => setSlidesTitle(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-amber-500 rounded placeholder-gray-750"
                      />
                    </div>
                    <p className="text-[8px] text-gray-550 leading-relaxed font-sans">
                      * Triggers the Google Slides API core engine to initialize a multi-tier slide module incorporating default layouts, ready for presentations.
                    </p>
                    <button 
                      onClick={handleCreateSlides}
                      className="w-full py-1.5 bg-amber-600 hover:bg-amber-500 text-white font-bold tracking-wider rounded uppercase transition-all"
                    >
                      BUILD PRESENTATION
                    </button>

                    <button 
                      onClick={handleBuildUnifiedCodex}
                      className="w-full py-1.5 bg-gradient-to-r from-amber-600 to-amber-800 hover:from-amber-500 hover:to-amber-700 text-white font-bold tracking-wider rounded uppercase transition-all border border-amber-950/40"
                    >
                      🔮 BUILD UNIFIED CODEX PRESENTATION (4-ACTS)
                    </button>
                  </div>
                </div>

                {/* Create Meet Space portal */}
                <div className="bg-black/20 border border-gray-850 p-3 rounded space-y-3 font-mono text-[10px]">
                  <p className="text-emerald-400 font-bold tracking-widest border-b border-gray-950 pb-1 flex items-center gap-1.5">
                    <Video className="w-3.5 h-3.5 text-emerald-400" />
                    <span>ALLOCATE INSTANT MEETING PORTAL (MEET)</span>
                  </p>
                  <div className="space-y-3 flex-1 flex flex-col justify-between">
                    <div>
                      <p className="text-gray-400 text-[10px] leading-relaxed font-sans">
                        Request instant virtual videoconference chambers directly through Google Meet API endpoint protocols. Returns secure copyable hyperlinks.
                      </p>
                      {createdMeetUrl && (
                        <div className="mt-3 bg-black/40 border border-emerald-950/50 p-2 rounded flex items-center justify-between text-emerald-300">
                          <span className="truncate mr-2 font-mono text-[9px]">{createdMeetUrl}</span>
                          <div className="flex shrink-0 gap-1.5">
                            <button 
                              onClick={() => {
                                navigator.clipboard.writeText(createdMeetUrl);
                                setSuccessMessage("Room link copied to clipboard!");
                                triggerVoiceUpdate("Room link cloned to clipboard buffer.");
                              }}
                              className="p-1 hover:bg-gray-800 rounded border border-gray-850 text-emerald-400 font-sans"
                            >
                              <Copy className="w-3 h-3" />
                            </button>
                            <a 
                              href={createdMeetUrl} 
                              target="_blank" 
                              rel="noreferrer"
                              className="p-1 hover:bg-emerald-950/20 rounded border border-emerald-900/35 text-emerald-400 font-sans"
                            >
                              <ExternalLink className="w-3 h-3" />
                            </a>
                          </div>
                        </div>
                      )}
                    </div>
                    <button 
                      onClick={handleCreateMeetSpace}
                      className="w-full py-1.5 bg-emerald-650 hover:bg-emerald-500 text-white font-bold tracking-wider rounded uppercase transition-all"
                    >
                      ALLOCATE MEET ROOM
                    </button>
                  </div>
                </div>

              </div>

              {/* 📓 NotebookLM Grounded Slide synthesis Suite */}
              <div className="bg-[#0f111a]/95 border border-gray-850 p-4 rounded-lg space-y-4 font-mono text-[10px] mt-4">
                <div className="flex items-center justify-between border-b border-gray-900 pb-2">
                  <p className="text-amber-500 font-bold tracking-widest flex items-center gap-2 text-[10px]">
                    <Presentation className="w-4 h-4 text-amber-400" />
                    <span>📓 NOTEBOOKLM DYNAMIC PRESENTATION BUILDER</span>
                  </p>
                  <span className="bg-amber-900/40 text-amber-400 border border-amber-900/70 px-2 py-0.5 rounded text-[8px] tracking-wider uppercase font-bold animate-pulse">
                    Grounded AI Design
                  </span>
                </div>

                <p className="text-gray-400 font-sans text-[11px] leading-relaxed">
                  Select your active research notes or Google Drive file assets below, enter your target presentation topic, and compile a tailored multi-theme Google Slides presentation instantly.
                </p>

                <div className="grid grid-cols-1 md:grid-cols-12 gap-4">
                  {/* Left side: select files & notes (7 cols) */}
                  <div className="md:col-span-7 space-y-3">
                    <div>
                      <span className="text-gray-500 block text-[9px] mb-1.5 font-bold uppercase tracking-wider">
                        1. CONFIGURE WORKSPACE GROUNDS (SELECT SOURCES)
                      </span>
                      
                      {/* Drive Files checklist */}
                      <div className="border border-gray-850 bg-black/35 rounded p-2.5 space-y-2">
                        <p className="text-[9px] text-gray-400 border-b border-gray-900 pb-1 font-bold tracking-wide uppercase">GOOGLE DRIVE FILES ({driveFiles.length} AVAILABLE)</p>
                        {driveFiles.length === 0 ? (
                          <p className="text-gray-500 text-[8px] uppercase font-sans py-2">No workspace file links synchronized.</p>
                        ) : (
                          <div className="max-h-[140px] overflow-y-auto space-y-1.5 pr-1 text-[9px]">
                            {driveFiles.map(file => {
                              const isChecked = selectedDriveFileIds.includes(file.id);
                              return (
                                <label key={file.id} className="flex items-start gap-2 max-w-full text-[9px] cursor-pointer hover:bg-white/5 p-1 rounded transition-colors truncate">
                                  <input 
                                    type="checkbox" 
                                    checked={isChecked}
                                    onChange={() => {
                                      if (isChecked) {
                                        setSelectedDriveFileIds(selectedDriveFileIds.filter(id => id !== file.id));
                                      } else {
                                        setSelectedDriveFileIds([...selectedDriveFileIds, file.id]);
                                      }
                                    }}
                                    className="mt-0.5 rounded border-gray-800 accent-amber-500 shrink-0" 
                                  />
                                  <div className="truncate flex items-center gap-1.5">
                                    {getFileIcon(file.mimeType)}
                                    <span className="text-gray-300 truncate font-sans" title={file.name}>{file.name}</span>
                                  </div>
                                </label>
                              );
                            })}
                          </div>
                        )}
                      </div>

                      {/* Keep Notes checklist */}
                      <div className="border border-gray-850 bg-black/35 rounded p-2.5 mt-2.5 space-y-2">
                        <p className="text-[9px] text-gray-400 border-b border-gray-900 pb-1 font-bold tracking-wide uppercase">MEMORY NOTES / KEEP NOTES ({keepNotes.length} AVAILABLE)</p>
                        {keepNotes.length === 0 ? (
                          <p className="text-gray-500 text-[8px] uppercase font-sans py-2">No localized Keep memory notes recorded.</p>
                        ) : (
                          <div className="max-h-[140px] overflow-y-auto space-y-1.5 pr-1 text-[9px]">
                            {keepNotes.map(note => {
                              const isChecked = selectedNoteIds.includes(note.id);
                              return (
                                <label key={note.id} className="flex items-start gap-2 text-[9px] cursor-pointer hover:bg-white/5 p-1 rounded transition-colors">
                                  <input 
                                    type="checkbox" 
                                    checked={isChecked}
                                    onChange={() => {
                                      if (isChecked) {
                                        setSelectedNoteIds(selectedNoteIds.filter(id => id !== note.id));
                                      } else {
                                        setSelectedNoteIds([...selectedNoteIds, note.id]);
                                      }
                                    }}
                                    className="mt-0.5 rounded border-gray-800 accent-amber-500 shrink-0" 
                                  />
                                  <div className="truncate flex flex-col font-sans mb-1">
                                    <span className="text-gray-300 font-semibold truncate text-[9px]">{note.title || "Untitled Note"}</span>
                                    <span className="text-gray-550 text-[8px] truncate mt-0.5">{note.content ? note.content.substring(0, 70) : "No content text..."}</span>
                                  </div>
                                </label>
                              );
                            })}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Right side: topic details & theme generator specifications (5 cols) */}
                  <div className="md:col-span-5 space-y-4 flex flex-col justify-between">
                    <div>
                      <span className="text-gray-505 block text-[9px] font-bold uppercase tracking-wider">
                        2. TARGET OBJECTIVE & THEME
                      </span>
                      
                      <div className="mt-2 space-y-3">
                        <div>
                          <label className="text-[8px] text-gray-400 block uppercase font-bold tracking-wide">Presentation Focus / Topic</label>
                          <input 
                            type="text" 
                            placeholder="e.g. Synthesis of Q3 Architectural Blueprints..."
                            value={nbLMTopic}
                            onChange={(e) => setNbLMTopic(e.target.value)}
                            className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-amber-500 rounded placeholder-gray-750 font-mono"
                          />
                        </div>

                        <div>
                          <label className="text-[8px] text-gray-400 block uppercase font-bold tracking-wide mb-1">Visual Design Aesthetic preset</label>
                          <div className="grid grid-cols-2 gap-1.5 text-[8px]">
                            {[
                              { id: "notebooklm_ivory", label: "NotebookLM Ivory", style: "bg-[#F9F8F6] text-gray-900 border-gray-400 hover:border-gray-500" },
                              { id: "cosmic_slate", label: "Cosmic Slate", style: "bg-[#0A0B10] text-gray-100 border-yellow-500/40 hover:border-yellow-400" },
                              { id: "emerald_cipher", label: "Emerald Cipher", style: "bg-[#020403] text-teal-400 border-emerald-500/40 hover:border-emerald-400" },
                              { id: "minimalist_clean", label: "Minimalist Clean", style: "bg-white text-black border-black/40 hover:border-black" }
                            ].map(themeOpt => (
                              <button
                                key={themeOpt.id}
                                type="button"
                                onClick={() => setNbLMTheme(themeOpt.id)}
                                className={`p-2 rounded border text-left font-bold transition-all truncate flex flex-col ${nbLMTheme === themeOpt.id ? "ring-2 ring-amber-500 border-amber-500 scale-102" : "opacity-75 hover:opacity-100"}`}
                              >
                                <span className="font-sans text-[8px]">{themeOpt.label}</span>
                                <span className="text-[7px] font-normal opacity-70 mt-0.5 truncate uppercase">
                                  {themeOpt.id.replace("_", " ")}
                                </span>
                              </button>
                            ))}
                          </div>
                        </div>
                      </div>
                    </div>

                    <div className="pt-4 border-t border-gray-900">
                      <div className="flex justify-between text-[8px] text-gray-500 mb-2 font-bold tracking-wider">
                        <span>SELECTED DOCUMENTS: {selectedDriveFileIds.length}</span>
                        <span>SELECTED MEMORIES: {selectedNoteIds.length}</span>
                      </div>
                      <button 
                        onClick={handleBuildNotebookLMSlides}
                        disabled={loading}
                        className="w-full py-2 bg-gradient-to-r from-amber-500 to-amber-700 hover:from-amber-400 hover:to-amber-600 text-white font-bold tracking-wider rounded uppercase transition-all shadow-md flex items-center justify-center gap-1.5 text-[10px]"
                      >
                        {loading ? (
                          <>
                            <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                            <span>COMPILING NOTEBOOKLM SUITE...</span>
                          </>
                        ) : (
                          <>
                            <Sparkles className="w-3.5 h-3.5 text-yellow-300" />
                            <span>🔮 COMPILE NOTEBOOKLM PRESENTATION</span>
                          </>
                        )}
                      </button>
                    </div>
                  </div>
                </div>
              </div>

            </div>
          )}

          {/* SERVICE VIEW 6: NOTES (KEEP) */}
          {activeService === "keep" && (
            <div className="space-y-4 animate-fadeIn">
              <div className="grid grid-cols-1 md:grid-cols-12 gap-4">
                
                {/* Create Note */}
                <div className="md:col-span-4 bg-black/20 border border-gray-850 p-3 rounded space-y-3 font-mono text-[10px]">
                  <p className="text-yellow-500 font-bold tracking-widest border-b border-gray-950 pb-1 flex items-center gap-1.5">
                    <StickyNote className="w-3.5 h-3.5 text-yellow-500" />
                    <span>SERIALIZE CORE NOTE (KEEP)</span>
                  </p>
                  <div className="space-y-2">
                    <div>
                      <span className="text-gray-500 block text-[9px]">NOTE THEME TITLE</span>
                      <input 
                        type="text" 
                        placeholder="e.g. Philosophical Synthesis" 
                        value={noteTitle} 
                        onChange={(e) => setNoteTitle(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-yellow-550 rounded placeholder-gray-750"
                      />
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[9px]">CORE MEMORY PAYLOAD</span>
                      <textarea 
                        rows={3}
                        placeholder="Type body ideas or lists..." 
                        value={noteContent} 
                        onChange={(e) => setNoteContent(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[9px] mt-1 text-gray-300 focus:outline-none focus:border-yellow-550 rounded placeholder-gray-750 font-sans"
                      />
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[8px]">TAGS (COMMA SEPARATED)</span>
                      <input 
                        type="text" 
                        placeholder="metaphysics, architecture" 
                        value={noteTags} 
                        onChange={(e) => setNoteTags(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[9px] mt-1 text-gray-300 focus:outline-none focus:border-yellow-550 rounded placeholder-gray-750"
                      />
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[8px] mb-1">VISUAL HUD ACCENT COLOR</span>
                      <div className="flex gap-2">
                        {["charcoal", "emerald", "blue", "indigo", "rose"].map((col) => (
                          <button 
                            key={col} 
                            onClick={() => setNoteColor(col)}
                            className={`w-3.5 h-3.5 rounded-full border transition-all ${
                              col === "charcoal" ? "bg-gray-800" :
                              col === "emerald" ? "bg-emerald-900" :
                              col === "blue" ? "bg-blue-900" :
                              col === "indigo" ? "bg-indigo-900" : "bg-red-910"
                            } ${noteColor === col ? "scale-125 border-yellow-400 ring-2 ring-yellow-400/20" : "border-gray-800 hover:scale-110"}`}
                          />
                        ))}
                      </div>
                    </div>
                    <button 
                      onClick={handleCreateKeepNote}
                      className="w-full py-1.5 bg-yellow-600 hover:bg-yellow-500 text-white font-bold tracking-wider rounded uppercase transition-all"
                    >
                      COMMIT NOTE TO DATABASE
                    </button>
                  </div>
                </div>

                {/* List Notes */}
                <div className="md:col-span-8 space-y-2">
                  <div className="flex flex-wrap justify-between items-center border-b border-gray-900 pb-1.5 gap-2">
                    <div className="flex flex-col gap-0.5">
                      <span className="text-[10px] text-gray-400 font-mono font-bold tracking-widest uppercase">Memory Board Logs ({keepNotes.length})</span>
                      {syncStatusDetail && (
                        <div className="text-[7.5px] font-mono text-gray-400 bg-black/35 border border-gray-850 px-1.5 py-0.5 rounded flex items-center gap-1">
                          <span className={`w-1 h-1 rounded-full ${
                            keepSyncStatus === "synced" ? "bg-emerald-400 animate-pulse" :
                            keepSyncStatus === "error" ? "bg-red-500" : "bg-yellow-450"
                          }`} />
                          <span className="font-bold text-gray-500">{keepSyncStatus.toUpperCase()}:</span>
                          <span className="text-gray-400 truncate max-w-[340px]">{syncStatusDetail}</span>
                        </div>
                      )}
                    </div>
                    <button onClick={fetchKeepNotes} className="text-gray-500 hover:text-cyan-400 transition-colors flex items-center gap-1 text-[8px] font-mono">
                      <RefreshCw className="w-2.5 h-2.5" />
                      <span>POLL STORED ENTITIES</span>
                    </button>
                  </div>

                  {loading ? (
                    <div className="flex items-center justify-center py-10 font-mono text-[10px] text-gray-500 gap-1.5 animate-pulse">
                      <RefreshCw className="w-4 h-4 animate-spin text-yellow-400" />
                      <span>SERIALIZING CLOUD DATASTORES...</span>
                    </div>
                  ) : keepNotes.length === 0 ? (
                    <p className="text-center py-12 font-mono text-[10px] text-gray-600 uppercase">No notes serialized to cloud cluster.</p>
                  ) : (
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-[280px] overflow-y-auto pr-1">
                      {keepNotes.map((note) => (
                        <div 
                          key={note.id} 
                          className={`border p-3 rounded relative group font-mono text-[10px] space-y-1.5 ${
                            note.color === "emerald" ? "bg-emerald-950/20 border-emerald-900/40 text-emerald-100" :
                            note.color === "blue" ? "bg-blue-950/20 border-blue-900/40 text-blue-100" :
                            note.color === "indigo" ? "bg-indigo-950/20 border-indigo-900/40 text-indigo-100" :
                            note.color === "rose" ? "bg-red-955/20 border-red-900/40 text-red-100" :
                            "bg-black/35 border-gray-850 text-gray-300"
                          }`}
                        >
                          <div className="flex justify-between items-start gap-2">
                            <span className="font-bold leading-tight block text-[11px] truncate w-[130px] pr-2 uppercase">{note.title}</span>
                            <button 
                              onClick={() => handleDeleteKeepNote(note.id, note.title)}
                              className="text-gray-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
                            >
                              <Trash2 className="w-3.5 h-3.5" />
                            </button>
                          </div>
                          <p className="text-gray-350 text-[9px] font-sans leading-relaxed whitespace-pre-wrap">{note.content}</p>
                          <div className="flex flex-wrap gap-1.5 pt-1.5 border-t border-black/10">
                            {note.tags.map((tag, idx) => (
                              <span key={idx} className="px-1.5 py-0.5 bg-black/20 rounded text-[7px] text-gray-400 uppercase tracking-wide font-mono">
                                #{tag}
                              </span>
                            ))}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>

              </div>
            </div>
          )}

          {/* SERVICE VIEW 7: CONTACTS */}
          {activeService === "contacts" && (
            <div className="space-y-4 animate-fadeIn">
              <div className="grid grid-cols-1 md:grid-cols-12 gap-4">
                
                {/* Create Contact */}
                <div className="md:col-span-5 bg-black/20 border border-gray-850 p-3 rounded space-y-3 font-mono text-[10px]">
                  <p className="text-cyan-400 font-bold tracking-widest border-b border-gray-950 pb-1 flex items-center gap-1.5">
                    <UserPlus className="w-3.5 h-3.5 text-cyan-400" />
                    <span>REGISTER NEW PROFILE CONTACT</span>
                  </p>
                  <div className="space-y-2">
                    <div className="grid grid-cols-2 gap-2">
                      <div>
                        <span className="text-gray-500 block text-[9px]">GIVEN NAME</span>
                        <input 
                          type="text" 
                          placeholder="Given Name" 
                          value={contactGivenName} 
                          onChange={(e) => setContactGivenName(e.target.value)}
                          className="bg-black/45 border border-gray-850 w-full p-1.5 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-cyan-500 rounded placeholder-gray-700"
                        />
                      </div>
                      <div>
                        <span className="text-gray-500 block text-[9px]">FAMILY NAME</span>
                        <input 
                          type="text" 
                          placeholder="Family Name" 
                          value={contactFamilyName} 
                          onChange={(e) => setContactFamilyName(e.target.value)}
                          className="bg-black/45 border border-gray-850 w-full p-1.5 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-cyan-500 rounded placeholder-gray-700"
                        />
                      </div>
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[9px]">EMAIL COMMUNICATION ADDRESS</span>
                      <input 
                        type="email" 
                        placeholder="recipient@domain.com" 
                        value={contactEmail} 
                        onChange={(e) => setContactEmail(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-cyan-500 rounded placeholder-gray-700"
                      />
                    </div>
                    <div>
                      <span className="text-gray-500 block text-[9px]">PHONE DESIGNATION PATH</span>
                      <input 
                        type="text" 
                        placeholder="e.g. +1 (555) 019-2834" 
                        value={contactPhone} 
                        onChange={(e) => setContactPhone(e.target.value)}
                        className="bg-black/45 border border-gray-850 w-full p-2 text-[10px] mt-1 text-gray-300 focus:outline-none focus:border-cyan-500 rounded placeholder-gray-700"
                      />
                    </div>
                    <button 
                      onClick={handleCreateContact}
                      className="w-full py-1.5 bg-cyan-600/80 hover:bg-cyan-500 text-white font-bold tracking-wider rounded uppercase transition-all"
                    >
                      REGISTER CONTACT CARD
                    </button>
                  </div>
                </div>

                {/* List Contacts */}
                <div className="md:col-span-7 space-y-2">
                  <div className="flex justify-between items-center border-b border-gray-900 pb-1">
                    <span className="text-[10px] text-gray-400 font-mono font-bold tracking-widest uppercase">Contact Database ({contacts.length})</span>
                    <button onClick={fetchContacts} className="text-gray-500 hover:text-cyan-400 transition-colors flex items-center gap-1 text-[8px] font-mono">
                      <RefreshCw className="w-2.5 h-2.5" />
                      <span>POLL CONTACT NODES</span>
                    </button>
                  </div>

                  {loading ? (
                    <div className="flex items-center justify-center py-10 font-mono text-[10px] text-gray-500 gap-1.5 animate-pulse">
                      <RefreshCw className="w-4 h-4 animate-spin text-cyan-400" />
                      <span>CATALOGING PEOPLES CONNECTIONS...</span>
                    </div>
                  ) : contacts.length === 0 ? (
                    <p className="text-center py-12 font-mono text-[10px] text-gray-600 uppercase">No people connections synced to contacts book.</p>
                  ) : (
                    <div className="max-h-[280px] overflow-y-auto space-y-1.5 pr-1.5">
                      {filteredContacts.map((contact) => (
                        <div key={contact.id} className="bg-black/20 hover:bg-black/45 border border-gray-850 p-2.5 rounded text-[10px] font-mono flex items-center justify-between gap-3 text-[10px] hover:border-gray-800 transition-colors">
                          <div className="flex items-center gap-2 truncate">
                            <div className="w-7 h-7 rounded bg-cyan-950/20 border border-cyan-900/30 flex items-center justify-center shrink-0">
                              <User className="w-4 h-4 text-cyan-400" />
                            </div>
                            <div className="truncate">
                              <span className="text-gray-200 font-bold block truncate leading-tight uppercase">{contact.name}</span>
                              <span className="text-gray-550 text-[8px] truncate block mt-0.5">{contact.email || "No email"} // {contact.phone || "No phone"}</span>
                            </div>
                          </div>
                          {contact.email && (
                            <button 
                              onClick={() => {
                                setMailTo(contact.email || "");
                                setActiveService("gmail");
                                triggerVoiceUpdate(`Ready to compose email transmission to ${contact.name}.`);
                              }}
                              className="py-1 px-2 hover:bg-red-950/15 border border-gray-850 hover:border-red-900/30 text-gray-400 hover:text-red-400 rounded text-[9px] font-mono"
                            >
                              EMAIL
                            </button>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>

              </div>
            </div>
          )}

          {/* SERVICE VIEW 8: GITHUB NEXUS */}
          {activeService === "github" && (
            <div className="space-y-4 animate-fadeIn">
              <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">

                {/* Configuration Console */}
                <div className="lg:col-span-4 bg-black/20 border border-violet-950/30 p-3 rounded space-y-3 font-mono text-[10px]">
                  <p className="text-violet-400 font-bold tracking-widest border-b border-violet-950/40 pb-1 flex items-center gap-1">
                    <Github className="w-3.5 h-3.5 text-violet-400" />
                    <span>GITHUB TELEMETRY PORTAL</span>
                  </p>

                  <div className="space-y-3">
                    <div>
                      <span className="text-gray-500 block text-[9px] uppercase font-semibold">Repository (owner/repo)</span>
                      <input 
                        type="text" 
                        value={githubRepo} 
                        onChange={(e) => setGithubRepo(e.target.value)}
                        placeholder="e.g. facebook/react" 
                        className="bg-black/45 border border-gray-850 w-full p-2 mt-1 text-gray-300 font-mono text-[10px] focus:outline-none focus:border-violet-500 rounded placeholder-gray-700"
                      />
                    </div>

                    <div>
                      <span className="text-gray-500 block text-[9px] uppercase font-semibold">Active branch / Ref</span>
                      <input 
                        type="text" 
                        value={githubBranch} 
                        onChange={(e) => setGithubBranch(e.target.value)}
                        placeholder="main" 
                        className="bg-black/45 border border-gray-850 w-full p-2 mt-1 text-gray-300 font-mono text-[10px] focus:outline-none focus:border-violet-500 rounded placeholder-gray-700"
                      />
                    </div>

                    <div>
                      <span className="text-gray-500 block text-[9px] uppercase font-semibold">Personal Access token (Optional)</span>
                      <input 
                        type="password" 
                        value={githubToken} 
                        onChange={(e) => setGithubToken(e.target.value)}
                        placeholder="ghp_XXXXXXXXXXXXXXXXXXXX" 
                        className="bg-black/45 border border-gray-850 w-full p-2 mt-1 text-gray-300 font-mono text-[10px] focus:outline-none focus:border-violet-500 rounded placeholder-gray-700"
                      />
                      <p className="text-[8px] text-gray-600 mt-1">Required only for accessing private repositories or to prevent API rate limits.</p>
                    </div>

                    <button 
                      onClick={() => fetchGithubRepoContents("")}
                      className="w-full py-1.5 bg-violet-800/80 hover:bg-violet-750 text-white font-bold tracking-wider rounded uppercase transition-all flex items-center justify-center gap-1 cursor-pointer"
                    >
                      <RefreshCw className="w-3 h-3 animate-pulse" />
                      <span>CONNECT & SCAN CORE</span>
                    </button>
                  </div>
                </div>

                {/* Directory Listings Console */}
                <div className="lg:col-span-8 space-y-2">
                  <div className="flex justify-between items-center border-b border-gray-900 pb-1">
                    <div className="flex items-center gap-1.5 font-mono text-[9px] text-gray-400">
                      <span className="font-bold tracking-widest uppercase">REPO CORE FILES:</span>
                      <span className="text-violet-400 bg-violet-950/20 px-1 border border-violet-900/30 rounded">{githubRepo}</span>
                    </div>
                    {githubPath && (
                      <button 
                        onClick={() => {
                          const parts = githubPath.split("/");
                          parts.pop();
                          fetchGithubRepoContents(parts.join("/"));
                        }}
                        className="text-gray-550 hover:text-white transition-colors text-[8px] font-mono border border-gray-850 px-1.5 rounded bg-black/10 hover:bg-black/30 cursor-pointer"
                      >
                        ◀ UP DIR
                      </button>
                    )}
                  </div>

                  {loadingGithub ? (
                    <div className="flex flex-col items-center justify-center py-12 font-mono text-[10px] text-gray-500 gap-2 animate-pulse">
                      <RefreshCw className="w-5 h-5 animate-spin text-violet-500" />
                      <span>COMMUNICATING WITH GITHUB CORE HOSTS...</span>
                    </div>
                  ) : githubFiles.length === 0 ? (
                    <div className="text-center py-12 font-mono text-[10px] text-gray-650 uppercase border border-dashed border-gray-850 rounded">
                      No tracked file directories connected. Tap Connect above to fetch indices.
                    </div>
                  ) : (
                    <div className="max-h-[300px] overflow-y-auto space-y-1 pr-1">
                      {githubFiles.map((file, idx) => (
                        <div key={idx} className="bg-black/15 hover:bg-black/30 border border-gray-850 p-2 rounded flex items-center justify-between gap-3 text-[10px] font-mono hover:border-violet-900/20 hover:bg-violet-950/5 transition-all">
                          <button
                            onClick={() => {
                              if (file.type === "dir") {
                                fetchGithubRepoContents(file.path);
                              }
                            }}
                            className="flex items-center gap-2 truncate text-left w-full outline-none cursor-pointer"
                          >
                            <span className="shrink-0">
                              {file.type === "dir" ? "📁" : "📄"}
                            </span>
                            <div className="truncate">
                              <span className={`${file.type === "dir" ? "text-violet-300 font-semibold" : "text-gray-300"} truncate block hover:underline`}>
                                {file.name}
                              </span>
                              <span className="text-gray-600 block text-[8px] truncate">
                                {file.type === "dir" ? "DIRECTORY" : `${(file.size / 1024).toFixed(1)} KB`}
                              </span>
                            </div>
                          </button>

                          {file.type === "file" && (
                            <button
                              onClick={() => handleLearnGithubFile(file)}
                              disabled={learningGithubPath !== null}
                              className={`p-1 px-1.5 rounded border border-violet-850 text-[8px] font-bold tracking-wider flex items-center gap-1 transition-all uppercase shrink-0 cursor-pointer ${
                                learningGithubPath === file.path
                                  ? "bg-violet-950/40 text-violet-400 animate-pulse border-violet-800"
                                  : "bg-violet-950/15 text-violet-300 hover:bg-violet-900/30 hover:border-violet-600"
                              }`}
                              title="Engage model logic on this codebase component to extract relations"
                            >
                              <Sparkles className="w-2.5 h-2.5 text-violet-400 animate-pulse" />
                              <span>{learningGithubPath === file.path ? "LEARNING..." : "LEARN CODE"}</span>
                            </button>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>

              </div>
            </div>
          )}

        </div>
      </div>

      {/* CONFIRMATION SAFETY WARNING MODAL */}
      {showConfirmModal && pendingAction && (
        <div className="fixed inset-0 bg-black/75 flex items-center justify-center z-50 p-4 font-mono animate-fadeIn">
          <div className="max-w-md w-full bg-[#0a0c10] border border-red-900/40 shadow-2xl p-4.5 rounded text-xs space-y-4">
            <div className="flex items-center gap-2 border-b border-red-950/45 pb-2 text-red-400">
              <AlertCircle className="w-4.5 h-4.5 animate-pulse shrink-0" />
              <span className="font-bold uppercase tracking-wider text-[11px]">ADMINISTRATIVE MUTATION VERIFICATION</span>
            </div>
            
            <div className="bg-red-950/10 border border-red-900/20 p-3 rounded">
              <span className="text-[9px] text-gray-500 uppercase block">Operation Action:</span>
              <p className="text-gray-200 font-bold uppercase mt-0.5 tracking-wide text-[10px]">{pendingAction.type}</p>
              
              <span className="text-[9px] text-gray-500 uppercase mt-2.5 block">Description Rule:</span>
              <p className="text-gray-350 italic mt-0.5 leading-relaxed font-sans text-[10px]">
                "{pendingAction.text}"
              </p>
            </div>

            <p className="text-[9px] text-gray-550 leading-relaxed font-sans">
              Boss, confirming this operation sends direct requests to your official Google account API nodes, mutating real cloud files. Do you authorize this transaction?
            </p>

            <div className="flex gap-2.5 pt-1 text-[10px] uppercase font-bold text-center">
              <button
                onClick={() => {
                  setShowConfirmModal(false);
                  setPendingAction(null);
                  triggerVoiceUpdate("Operation averted.");
                }}
                className="flex-1 py-1.5 bg-gray-900 hover:bg-gray-800 text-gray-400 border border-gray-800 rounded tracking-wider"
              >
                ABORT / CANCEL
              </button>
              <button
                onClick={handleExecutePendingAction}
                className="flex-1 py-1.5 bg-red-650 hover:bg-red-500 text-white rounded tracking-wider shadow-lg shadow-red-500/15"
              >
                AUTHORIZE / EXECUTE
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
