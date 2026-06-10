export interface Snippet {
  id: string;
  title: string;
  command: string;
  description: string | null;
  tags: string[];
  createdAt: string;
  updatedAt: string;
}

export interface SnippetInput {
  title: string;
  command: string;
  description: string | null;
  tags: string[];
}
