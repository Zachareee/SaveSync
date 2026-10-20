import { Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle } from "@suid/material";
import { Component, JSX } from "solid-js";

const PluginDialog: Component<JSX.HTMLAttributes<HTMLDivElement>
  & { filepath: string }
  & Record<"addPlugin" | "reset", () => void>
>
  = ({ filepath, addPlugin, reset }) => {
    return <>
      <Dialog open={filepath?.length != 0}>
        <DialogTitle>Are you sure you want to add this plugin?</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Plugins may be created with malicious intent which may compromise your computer. Only add a plugin if you trust the author of the plugin and its contents
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={addPlugin}>Add</Button>
          <Button onClick={reset}>Cancel</Button>
        </DialogActions>
      </Dialog>
    </>
  }

export default PluginDialog;
