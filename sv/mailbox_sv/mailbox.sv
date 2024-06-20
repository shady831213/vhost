`ifndef __MAILBOX_SV__
 `define __MAILBOX_SV__
 `include "mailbox.svh"

import "DPI-C" function void mb_get_space(string ch_name, output string space_name);
import "DPI-C" context function void mb_backdoor_write_u8(string space_name, `MB_PTR addr, byte unsigned data);
import "DPI-C" context function void mb_backdoor_read_u8(string space_name, `MB_PTR addr, output byte unsigned data);
import "DPI-C" context function void mb_backdoor_write_u16(string space_name, `MB_PTR addr, shortint unsigned data);
import "DPI-C" context function void mb_backdoor_read_u16(string space_name, `MB_PTR addr, output shortint unsigned data);
import "DPI-C" context function void mb_backdoor_write_u32(string space_name, `MB_PTR addr, int unsigned data);
import "DPI-C" context function void mb_backdoor_read_u32(string space_name, `MB_PTR addr, output int unsigned data);
import "DPI-C" context function void mb_backdoor_write_u64(string space_name, `MB_PTR addr, `MB_PTR data);
import "DPI-C" context function void mb_backdoor_read_u64(string space_name, `MB_PTR addr, output `MB_PTR data);
import "DPI-C" context function void mb_backdoor_write_string(string space_name, `MB_PTR addr, string data);
import "DPI-C" context function void mb_backdoor_read_string(string space_name, `MB_PTR addr, output string data);
import "DPI-C" context task mb_server_run();

export "DPI-C" function sv_registery_ch;
export "DPI-C" function mb_exit;
export "DPI-C" task mb_step;
export "DPI-C" function mb_sv_call;
export "DPI-C" function mb_bb_mem_write;
export "DPI-C" function mb_bb_mem_read;

typedef class Mailbox;
typedef class MbRoot;
typedef class MbBBMemFactory;

function automatic void sv_registery_ch(string ch_name);
   MbRoot::mb().registery_ch(ch_name);
endfunction

function automatic void mb_exit(int unsigned code);
   MbRoot::mb().exit(code);
endfunction

task automatic mb_step();
   MbRoot::mb().step();
endtask

function automatic `MB_PTR mb_sv_call(string ch_name, string method, int unsigned arg_len, `MB_PTR args[`SV_CALL_MAX_ARGS], output int unsigned status);
   return MbRoot::mb().poll(ch_name, method, arg_len, args, status);
endfunction

function automatic void mb_bb_mem_write(string name, `MB_PTR addr, byte unsigned data);
   MbBBMemFactory::get().write(name, addr, data);
endfunction

function automatic void mb_bb_mem_read(string name, `MB_PTR addr, output byte unsigned data);
   MbBBMemFactory::get().read(name, addr, data);
endfunction

class MbRoot;
   local static Mailbox _mb;
   static function Mailbox mb();
      if (_mb == null) begin
         _mb = new();
      end
      return _mb;
   endfunction
endclass

virtual class MbBBMem;
   string name;
   function new(string name);
      this.name = name;
   endfunction

   pure virtual function void write(`MB_PTR addr, byte unsigned data);
   pure virtual function void read(`MB_PTR addr, output byte unsigned data);
endclass

class MbBBMemFactory;
   local static MbBBMemFactory inst;
   local MbBBMem mems[string];
   static function MbBBMemFactory get();
      if (inst == null) begin
         inst = new();
      end
      return inst;
   endfunction

   local function new();
   endfunction

   function void registery(MbBBMem mem);
      if (this.mems.exists(mem.name)) begin
         `uvm_fatal("MbBBMemFactory", $sformatf("%s has been reisteryed!", mem.name));
      end
      mems[mem.name] = mem;
   endfunction

   function void write(string name, int unsigned addr, byte unsigned data);
      if (!this.mems.exists(name)) begin
         `uvm_fatal("MbBBMemFactory", $sformatf("%s does not exist!", name));
      end
      mems[name].write(addr, data);
   endfunction

   function void read(string name, int unsigned addr, output byte unsigned data);
      if (!this.mems.exists(name)) begin
         `uvm_fatal("MbBBMemFactory", $sformatf("%s does not exist!", name));
      end
      mems[name].read(addr, data);
   endfunction
endclass

virtual class MailboxSvCall extends uvm_object;
   typedef enum bit[1:0] {READY, PENDING, INIT} MailboxSvCallState;
   local MailboxSvCallState state;
   local bit [31:0] ret;
   local uvm_event next;


  `uvm_field_utils_begin(MailboxSvCall)
  `uvm_field_utils_end

  function new (string name = "MailboxSvCall");
     super.new(name);
     this.state = INIT;
     this.next = new();
  endfunction

  function int unsigned poll(string space, int unsigned arg_len, `MB_PTR args[], output int unsigned status);
     case(this.state)
       INIT: begin
          this.parse_args(space, arg_len, args);
          status = 1;
          this.next.trigger();
          return 0;
       end
       READY: begin
          bit[31:0] ret = this.ret;
          status = 0;
          this.state = INIT;
          return this.ret;
       end
       PENDING: begin
          status = 1;
          return 0;
       end
     endcase
  endfunction // poll

  pure virtual protected function void parse_args(string space, bit[31:0] arg_len, `MB_PTR args[]);

  pure virtual protected task execute();

  task run();
     fork
        begin
           this.next.wait_on();
           this.next.reset();
           this.state = PENDING;
           forever begin
              this.execute();
           end
        end
     join_none;
  endtask // run

  protected task yield(bit[31:0] result);
     this.ret = result;
     this.state = READY;
     this.next.wait_on();
     this.next.reset();
     this.state = PENDING;
  endtask

endclass // MailboxSvCall

class MailboxCh extends uvm_object;
   protected MailboxSvCall methods[string];
   protected string space;

   `uvm_object_utils(MailboxCh)

  function new(string name="MailboxCh");
     super.new(name);
     mb_get_space(name, this.space);
  endfunction // new

  function void registery(MailboxSvCall method);
     MailboxSvCall _method;
     if (this.methods.exists(method.get_name())) begin
        `uvm_fatal(this.get_full_name(), $psprintf("%s already exists!", method.get_name()));
     end
     $cast(_method, method.clone());
     this.methods[method.get_name()] = _method;
  endfunction // registery

  task start();
     foreach(this.methods[i]) begin
        this.methods[i].run();
     end
  endtask


  function int unsigned poll(string method, int unsigned arg_len, `MB_PTR args[], output int unsigned status);
     if (!this.methods.exists(method)) begin
        `uvm_fatal(this.get_full_name(), $psprintf("%s does not exist!", method));
     end
     return this.methods[method].poll(space, arg_len, args, status);
  endfunction
endclass

class Mailbox extends uvm_object;
   protected uvm_event finish_e;
   protected bit[31:0] exit_status;
   protected MailboxCh chs[string];
   protected MailboxSvCall methods[string];
   protected virtual MailboxIf inf;

   `uvm_object_utils(Mailbox)

  function new(string name="Mailbox");
     super.new(name);
     this.finish_e = new();
  endfunction // new

  function void exit(bit[31:0] code);
     this.exit_status = code;
     this.finish_e.trigger();
  endfunction

  function void set_inf(virtual MailboxIf inf);
     this.inf = inf;
  endfunction

  task step();
     @(posedge inf.clk);
  endtask

  task wait_finish();
     this.finish_e.wait_on();
     if (this.exit_status == 0) begin
        `uvm_info(this.get_name(), "exit 0!", UVM_LOW);
     end
     else begin
        `uvm_error(this.get_name(), $psprintf("exit %0d!", this.exit_status));
     end
  endtask

  function void registery_ch(string ch_name);
     if (this.chs.exists(ch_name)) begin
        `uvm_fatal(this.get_full_name(), $psprintf("%s already exists!", ch_name));
     end
     this.chs[ch_name] = MailboxCh::type_id::create(ch_name);
  endfunction // registery

  function void registery_method(MailboxSvCall method);
      `uvm_info(this.get_full_name(), $psprintf("registery_method %s!", method.get_name()), UVM_LOW);
     this.methods[method.get_name()] = method;
  endfunction // registery_method

  task start();
     wait(this.chs.size() != 0);
     foreach(this.chs[i]) begin
        foreach(this.methods[j]) begin
            `uvm_info(this.get_full_name(), $psprintf("ch %s method %s!",i, j), UVM_LOW);
            chs[i].registery(this.methods[j]);
        end
        chs[i].start();
     end
  endtask

  function int unsigned poll(string ch_name, string method, int unsigned arg_len, `MB_PTR args[], output int unsigned status);
     if (!this.chs.exists(ch_name)) begin
        `uvm_fatal(this.get_full_name(), $psprintf("%s does not exist!", ch_name));
     end
     return this.chs[ch_name].poll(method, arg_len, args, status);
  endfunction

endclass
`endif
